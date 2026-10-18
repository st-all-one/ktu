//! Integração do canal semântico (H8): `ktu memory drain` contra um servidor falso.
//!
//! O servidor de embeddings é um HTTP mínimo em `127.0.0.1` — o gate **nunca** fala com
//! um `llama-server` real (o determinismo vive nos *fakes* de `ktu-core`).

use std::fs;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;

static COUNTER: AtomicU64 = AtomicU64::new(0);

/// Raiz temporária única por teste (o processo é partilhado).
fn fresh_root(tag: &str) -> PathBuf {
    let id = COUNTER.fetch_add(1, Ordering::Relaxed);
    let root = std::env::temp_dir().join(format!("ktu-embed-{tag}-{}-{id}", std::process::id()));
    let _ignored = fs::remove_dir_all(&root);
    root
}

/// Servidor HTTP mínimo que responde a `POST /v1/embeddings` com um vetor por texto.
fn spawn_server(dimensions: usize) -> io::Result<(String, thread::JoinHandle<()>)> {
    let listener = TcpListener::bind("127.0.0.1:0")?;
    let port = listener.local_addr()?.port();
    let endpoint = format!("http://127.0.0.1:{port}/v1/embeddings");
    let handle = thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(mut stream) = stream else { break };
            let mut reader = BufReader::new(match stream.try_clone() {
                Ok(clone) => clone,
                Err(_) => break,
            });
            let mut length = 0_usize;
            loop {
                let mut line = String::new();
                if reader.read_line(&mut line).unwrap_or(0) == 0 || line == "\r\n" {
                    break;
                }
                if let Some(value) = line.to_ascii_lowercase().strip_prefix("content-length:") {
                    length = value.trim().parse().unwrap_or(0);
                }
            }
            let mut body = vec![0_u8; length];
            let _ignored = reader.read_exact(&mut body);
            let count = serde_json::from_slice::<serde_json::Value>(&body)
                .ok()
                .and_then(|value| {
                    value
                        .get("input")
                        .and_then(serde_json::Value::as_array)
                        .map(Vec::len)
                })
                .unwrap_or(1);
            let items: Vec<serde_json::Value> = (0..count)
                .map(|index| serde_json::json!({ "index": index, "embedding": vec![0.5_f64; dimensions] }))
                .collect();
            let payload = serde_json::json!({ "data": items }).to_string();
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{payload}",
                payload.len()
            );
            let _ignored = stream.write_all(response.as_bytes());
            let _ignored = stream.flush();
        }
    });
    Ok((endpoint, handle))
}

/// Nota válida no formato JSONL do motor (o contrato do knudge).
const NOTE: &str = concat!(
    r#"{"frontmatter":{"id":"fact_00abc123","type":"fact","#,
    r#""statement":"o dedup usa o body_hash","created_at":1700000000000,"#,
    r#""body_hash":"deadbeef","schema_version":2},"body":"corpo da nota"}"#,
    "\n"
);

/// Escreve a memória do projecto (notas + `memory.toml` apontado ao servidor).
fn seed(root: &Path, endpoint: &str) -> Result<(), Box<dyn std::error::Error>> {
    let memory_dir = root.join(".agents/memory");
    fs::create_dir_all(&memory_dir)?;
    fs::write(memory_dir.join("notes.jsonl"), NOTE)?;
    fs::write(
        root.join(".agents/memory.toml"),
        format!(
            "[embeddings]\nprovider = \"http\"\nendpoint = \"{endpoint}\"\ndimensions = 4\nprefix = \"none\"\n"
        ),
    )?;
    Ok(())
}

/// Corre `ktu <args>` no `root`, isolado da config do utilizador.
fn ktu(root: &Path, args: &[&str]) -> Result<Output, Box<dyn std::error::Error>> {
    Ok(Command::new(env!("CARGO_BIN_EXE_ktu"))
        .args(args)
        .env("XDG_CONFIG_HOME", root.join("xdg"))
        .env("XDG_CACHE_HOME", root.join("cache"))
        .env("HOME", root.join("home"))
        .current_dir(root)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()?)
}

/// `stdout` como texto.
fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

/// Valor de `memory.embeddings.<key>` na saída de texto.
fn field(text: &str, key: &str) -> Option<String> {
    text.lines()
        .find_map(|line| line.strip_prefix(&format!("memory.embeddings.{key}: ")))
        .map(str::to_owned)
}

#[test]
fn digest_indexes_the_queue_and_status_reports_it() -> Result<(), Box<dyn std::error::Error>> {
    let root = fresh_root("digest");
    fs::create_dir_all(&root)?;
    let (endpoint, _server) = spawn_server(4)?;
    seed(&root, &endpoint)?;

    let cold = ktu(&root, &["memory", "drain", "--status"])?;
    assert!(cold.status.success(), "{cold:?}");
    let cold = stdout(&cold);
    assert_eq!(field(&cold, "channel").as_deref(), Some("semantic"));
    assert_eq!(field(&cold, "index").as_deref(), Some("absent"));
    assert_eq!(field(&cold, "pending").as_deref(), Some("1"));

    let digest = ktu(&root, &["memory", "drain", "--digest"])?;
    assert!(digest.status.success(), "{digest:?}");
    let text = stdout(&digest);
    assert_eq!(field(&text, "indexed").as_deref(), Some("1"));
    assert_eq!(field(&text, "pending").as_deref(), Some("0"));
    assert!(root.join(".agents/memory/.idx/embeddings.jsonl").exists());

    let warm = ktu(&root, &["memory", "drain", "--status"])?;
    let warm = stdout(&warm);
    assert_eq!(field(&warm, "index").as_deref(), Some("present"));
    assert_eq!(field(&warm, "indexed").as_deref(), Some("1"));
    assert_eq!(field(&warm, "pending").as_deref(), Some("0"));
    assert_eq!(field(&warm, "next").as_deref(), Some("fila em dia"));

    // Idempotente: nada mudou, nada a digerir.
    let again = ktu(&root, &["memory", "drain", "--digest", "--json"])?;
    let data: serde_json::Value = serde_json::from_slice(&again.stdout)?;
    assert_eq!(
        data.get("data").and_then(|value| value.get("indexed")),
        Some(&serde_json::json!(0))
    );

    // `--force` reindexa tudo do zero.
    let forced = ktu(&root, &["memory", "drain", "--force"])?;
    assert!(forced.status.success(), "{forced:?}");
    assert_eq!(field(&stdout(&forced), "indexed").as_deref(), Some("1"));
    let _ignored = fs::remove_dir_all(&root);
    Ok(())
}

#[test]
fn digest_without_a_server_fails_but_keeps_the_queue() -> Result<(), Box<dyn std::error::Error>> {
    let root = fresh_root("noserver");
    fs::create_dir_all(&root)?;
    // Porta 1 (fechada): o provedor está indisponível.
    seed(&root, "http://127.0.0.1:1/v1/embeddings")?;

    let out = ktu(&root, &["memory", "drain", "--digest"])?;
    assert_eq!(out.status.code(), Some(1), "{out:?}");
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("indisponivel"),
        "{out:?}"
    );
    let after = ktu(&root, &["memory", "drain", "--status"])?;
    let after = stdout(&after);
    assert_eq!(
        field(&after, "pending").as_deref(),
        Some("1"),
        "a nota fica pendente"
    );
    assert_eq!(field(&after, "index").as_deref(), Some("absent"));
    let _ignored = fs::remove_dir_all(&root);
    Ok(())
}

#[test]
fn provider_none_is_lexical_and_digest_is_a_noop() -> Result<(), Box<dyn std::error::Error>> {
    let root = fresh_root("lexical");
    fs::create_dir_all(&root)?;
    let memory_dir = root.join(".agents/memory");
    fs::create_dir_all(&memory_dir)?;
    fs::write(memory_dir.join("notes.jsonl"), NOTE)?;

    let out = ktu(&root, &["memory", "drain", "--status"])?;
    assert!(out.status.success(), "{out:?}");
    let text = stdout(&out);
    assert_eq!(field(&text, "channel").as_deref(), Some("lexical"));
    assert_eq!(field(&text, "pending").as_deref(), Some("0"));

    let digest = ktu(&root, &["memory", "drain", "--digest"])?;
    assert!(digest.status.success(), "{digest:?}");
    assert!(stdout(&digest).contains("lexical"));
    assert!(
        !memory_dir.join(".idx").exists(),
        "nada derivado foi escrito"
    );
    let _ignored = fs::remove_dir_all(&root);
    Ok(())
}

#[test]
fn invalid_memory_toml_fails_high() -> Result<(), Box<dyn std::error::Error>> {
    let root = fresh_root("schema");
    fs::create_dir_all(root.join(".agents"))?;
    fs::write(root.join(".agents/memory.toml"), "[embeddings]\nnope = 1\n")?;
    let out = ktu(&root, &["memory", "drain", "--status"])?;
    assert_eq!(out.status.code(), Some(1), "{out:?}");
    assert!(String::from_utf8_lossy(&out.stderr).contains("memory.toml"));
    let _ignored = fs::remove_dir_all(&root);
    Ok(())
}

#[test]
fn worker_defines_every_action() -> Result<(), Box<dyn std::error::Error>> {
    // O worker é embutido: um heredoc mal terminado apagaria funções em silêncio.
    let script = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/ktu-idle.sh");
    let out = Command::new("bash")
        .arg("-c")
        .arg(format!(
            "source '{}' >/dev/null 2>&1; declare -F",
            script.display()
        ))
        .output()?;
    let functions = String::from_utf8_lossy(&out.stdout).into_owned();
    for name in [
        "install_worker",
        "do_subscribe",
        "do_unsubscribe",
        "do_status",
        "do_uninstall",
        "do_reconcile",
        "do_run",
        "drain_all",
        "ensure_server",
        "ensure_model",
        "align_memory_toml",
        "detect_scheduler",
        "projects_list",
        "read_settings",
        "write_conf",
        "trash",
    ] {
        assert!(functions.contains(name), "funcao ausente no worker: {name}");
    }
    Ok(())
}

#[test]
fn service_status_reports_the_worker_without_errors() -> Result<(), Box<dyn std::error::Error>> {
    let root = fresh_root("status");
    fs::create_dir_all(&root)?;
    let out = ktu(&root, &["memory", "drain", "service", "--status"])?;
    assert!(out.status.success(), "{out:?}");
    let text = stdout(&out);
    assert!(text.contains("worker: ktu-idle"), "{text}");
    assert!(text.contains("config:"), "{text}");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        !stderr.contains("not found") && !stderr.contains("nao encontrado"),
        "{stderr}"
    );
    let _ignored = fs::remove_dir_all(&root);
    Ok(())
}

#[test]
fn service_dry_run_plans_without_touching_the_system() -> Result<(), Box<dyn std::error::Error>> {
    let root = fresh_root("service");
    fs::create_dir_all(&root)?;
    let out = ktu(
        &root,
        &["memory", "drain", "service", "--install", "--dry-run"],
    )?;
    assert!(out.status.success(), "{out:?}");
    let text = stdout(&out);
    assert!(text.contains("dry-run"), "{text}");
    assert!(text.contains("embeddinggemma-2-Q8_0.gguf"), "{text}");
    assert!(
        !root.join("xdg").join("ktu").join("idle.conf").exists(),
        "dry-run nao escreve configuracao"
    );
    assert!(
        !root.join("xdg").join("systemd").exists(),
        "dry-run nao escreve unidades"
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        !stderr.contains("not found") && !stderr.contains("nao encontrado"),
        "{stderr}"
    );
    let _ignored = fs::remove_dir_all(&root);
    Ok(())
}

#[test]
fn uninstall_moves_the_model_to_the_trash() -> Result<(), Box<dyn std::error::Error>> {
    let root = fresh_root("trash");
    fs::create_dir_all(&root)?;
    let model = root
        .join("xdg")
        .join("ktu")
        .join("embeddinggemma-2-Q8_0.gguf");
    fs::create_dir_all(model.parent().unwrap_or(&root))?;
    fs::write(&model, b"pesos")?;

    let out = ktu(
        &root,
        &[
            "memory",
            "drain",
            "service",
            "--uninstall",
            "--remove-model",
        ],
    )?;
    assert!(out.status.success(), "{out:?}");
    assert!(!model.exists(), "o modelo saiu do lugar");
    let trash = root.join("cache").join("ktu").join("trash");
    let moved = fs::read_dir(&trash)?.count();
    assert_eq!(moved, 1, "foi para o lixo recuperavel: {trash:?}");
    let _ignored = fs::remove_dir_all(&root);
    Ok(())
}
