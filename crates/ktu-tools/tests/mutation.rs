//! Integração da fila de mutação por caminho (F4) com as tools reais.
//!
//! Exercita o caminho completo (`write_file`/`edit_file` → `resolve_*` → fila →
//! `StdFs`) com **N mutações concorrentes** ao mesmo ficheiro.

use std::fmt::Write as _;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use ktu_core::domain::{ToolCall, ToolCallId};
use ktu_core::tools::ToolExecutor;
use ktu_tools::{FileMutationQueue, FsTools, StdFs};

static COUNTER: AtomicU64 = AtomicU64::new(0);

/// Raiz temporária única por teste.
fn fresh_root(tag: &str) -> PathBuf {
    let id = COUNTER.fetch_add(1, Ordering::Relaxed);
    let root = std::env::temp_dir().join(format!("ktu-mut-{tag}-{}-{id}", std::process::id()));
    let _ignored = std::fs::remove_dir_all(&root);
    root
}

/// Chamada de tool com argumentos JSON.
fn call(name: &str, arguments: &str) -> ToolCall {
    ToolCall {
        id: ToolCallId::new("c1"),
        name: name.to_owned(),
        arguments: arguments.to_owned(),
    }
}

/// N `write_file` concorrentes ao mesmo caminho: o ficheiro final é **um** dos escritos.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn concurrent_writes_to_the_same_path_never_interleave() -> Result<(), String> {
    let root = fresh_root("same");
    std::fs::create_dir_all(&root).map_err(|error| error.to_string())?;
    let fs: &'static StdFs = Box::leak(Box::new(StdFs));
    // A fila e a porta vivem o teste inteiro (as tarefas são `'static`).
    let mutations: &'static FileMutationQueue = Box::leak(Box::new(FileMutationQueue::new()));
    let tools = Arc::new(FsTools::new(root.clone(), fs, mutations));

    // Conteúdos distintos e grandes o suficiente para uma escrita intercalada se notar.
    let contents: Vec<String> = (0..8)
        .map(|index| format!("{}#{index}\n", "linha-de-preenchimento".repeat(400)))
        .collect();
    let mut tasks = Vec::new();
    for content in &contents {
        let tools = Arc::clone(&tools);
        let arguments = serde_json::json!({ "path": "alvo.txt", "content": content }).to_string();
        tasks.push(tokio::spawn(async move {
            let outcome = tools.execute(&call("write_file", &arguments)).await;
            outcome.is_error
        }));
    }
    for task in tasks {
        let failed = task.await.map_err(|error| error.to_string())?;
        assert!(!failed, "uma escrita falhou");
    }

    let final_text = std::fs::read_to_string(root.join("alvo.txt")).map_err(|e| e.to_string())?;
    assert!(
        contents.iter().any(|content| content == &final_text),
        "o ficheiro final nao e nenhum dos escritos ({} bytes)",
        final_text.len()
    );
    let _ignored = std::fs::remove_dir_all(&root);
    Ok(())
}

/// N `edit_file` concorrentes ao mesmo ficheiro: **nenhuma** alteração se perde.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn concurrent_edits_to_the_same_path_lose_nothing() -> Result<(), String> {
    let root = fresh_root("edits");
    std::fs::create_dir_all(&root).map_err(|error| error.to_string())?;
    let mut initial = String::new();
    for index in 0..8 {
        let _ignored = writeln!(initial, "marca-{index}");
    }
    std::fs::write(root.join("alvo.txt"), &initial).map_err(|error| error.to_string())?;
    let fs: &'static StdFs = Box::leak(Box::new(StdFs));
    // A fila e a porta vivem o teste inteiro (as tarefas são `'static`).
    let mutations: &'static FileMutationQueue = Box::leak(Box::new(FileMutationQueue::new()));
    let tools = Arc::new(FsTools::new(root.clone(), fs, mutations));

    let mut tasks = Vec::new();
    for index in 0..8 {
        let tools = Arc::clone(&tools);
        let arguments = serde_json::json!({
            "path": "alvo.txt",
            "edits": [{ "old": format!("marca-{index}\n"), "new": format!("editada-{index}\n") }],
        })
        .to_string();
        tasks.push(tokio::spawn(async move {
            let outcome = tools.execute(&call("edit_file", &arguments)).await;
            outcome.is_error
        }));
    }
    for task in tasks {
        let failed = task.await.map_err(|error| error.to_string())?;
        assert!(!failed, "uma edicao falhou");
    }

    let final_text = std::fs::read_to_string(root.join("alvo.txt")).map_err(|e| e.to_string())?;
    // Se uma edicao tivesse lido fora da fila, a gravacao seguinte apagava-a.
    for index in 0..8 {
        assert!(
            final_text.contains(&format!("editada-{index}\n")),
            "perdeu a edicao {index}:\n{final_text}"
        );
    }
    let _ignored = std::fs::remove_dir_all(&root);
    Ok(())
}

/// Dois conjuntos de tools (dois turnos/sessões) **partilhando** a fila: as mutações
/// ao mesmo ficheiro serializam entre os dois — e a chave e uma so.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn two_tool_sets_sharing_a_queue_serialize_the_same_path() -> Result<(), String> {
    let root = fresh_root("partilha");
    std::fs::create_dir_all(&root).map_err(|error| error.to_string())?;
    let mut initial = String::new();
    for index in 0..8 {
        let _ignored = writeln!(initial, "marca-{index}");
    }
    std::fs::write(root.join("alvo.txt"), &initial).map_err(|error| error.to_string())?;
    let fs: &'static StdFs = Box::leak(Box::new(StdFs));
    let mutations: &'static FileMutationQueue = Box::leak(Box::new(FileMutationQueue::new()));
    // Duas instâncias distintas — como dois turnos (duas T2) do mesmo processo.
    let first = Arc::new(FsTools::new(root.clone(), fs, mutations));
    let second = Arc::new(FsTools::new(root.clone(), fs, mutations));

    let mut tasks = Vec::new();
    for (index, tools) in [(0, Arc::clone(&first)), (1, Arc::clone(&second))] {
        let arguments = serde_json::json!({
            "path": "alvo.txt",
            "edits": [{ "old": format!("marca-{index}\n"), "new": format!("editada-{index}\n") }],
        })
        .to_string();
        tasks.push(tokio::spawn(async move {
            tools.execute(&call("edit_file", &arguments)).await.is_error
        }));
    }
    for task in tasks {
        let failed = task.await.map_err(|error| error.to_string())?;
        assert!(!failed, "uma edicao falhou");
    }

    let final_text = std::fs::read_to_string(root.join("alvo.txt")).map_err(|e| e.to_string())?;
    assert!(final_text.contains("editada-0\n"), "{final_text}");
    assert!(final_text.contains("editada-1\n"), "{final_text}");
    // Uma só entrada na fila: as duas instâncias partilham a mesma chave.
    assert_eq!(mutations.tracked(), 1, "as instancias nao partilham a fila");
    let _ignored = std::fs::remove_dir_all(&root);
    Ok(())
}
