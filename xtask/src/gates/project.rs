//! Gates determinísticos **project-bound** e de **memória** (G14, G15).

use std::path::Path;

use super::support::{Ctx, Gate, count, exec, hash, read, stdout, summary_line, write};

/// G14 — project-bound: `init` idempotente, git reversível, precedência, M3 e M5.
pub(crate) fn g14(ctx: &Ctx) -> Result<Gate, String> {
    let mut gate = Gate::new(
        "G14",
        "Project-bound: init idempotente, git, precedencia, M3/M5",
    );
    let dir = ctx.fresh("g14")?;
    let git = exec(Path::new("git"), &["init", "-q"], &dir, "")?;
    gate.check(
        "git init disponivel (o cenário exige um repositorio)",
        git.status.success(),
        git.status.to_string(),
    );
    let first = ctx.ktu(&dir, &["init"], "")?;
    gate.check(
        "ktu init corre",
        first.status.success(),
        first.status.to_string(),
    );
    let managed = [".git/info/exclude", ".gitattributes"];
    let before: Vec<String> = managed
        .iter()
        .map(|path| read(&dir.join(path)).map(|text| hash(&text)))
        .collect::<Result<_, _>>()?;
    let second = ctx.ktu(&dir, &["init"], "")?;
    let after: Vec<String> = managed
        .iter()
        .map(|path| read(&dir.join(path)).map(|text| hash(&text)))
        .collect::<Result<_, _>>()?;
    gate.check(
        "ktu init e idempotente (ficheiros geridos inalterados)",
        second.status.success() && before == after,
        format!("antes={before:?} depois={after:?}"),
    );
    let exclude = read(&dir.join(".git/info/exclude"))?;
    let attributes = read(&dir.join(".gitattributes"))?;
    gate.check(
        "marcadores sem duplicacao (bloco unico)",
        count(&attributes, "# ktu:start") == 1 && count(&exclude, "/.ktu/") == 1,
        format!(
            "attributes={} exclude /.ktu/={}",
            count(&attributes, "# ktu:start"),
            count(&exclude, "/.ktu/")
        ),
    );
    gate.check(
        "por omissao o conhecimento versiona-se (so o derivado e excluido)",
        exclude.contains("/.agents/memory/index/") && !exclude.contains("/.agents/\n"),
        exclude.lines().last().unwrap_or_default().to_owned(),
    );
    // Reversibilidade: `--git-excluded` exclui todo o `.agents/`; `--git-tracked` volta atras.
    let excluded = ctx.ktu(&dir, &["init", "--git-excluded"], "")?;
    let exclude_excluded = read(&dir.join(".git/info/exclude"))?;
    gate.check(
        "--git-excluded exclui todo o .agents/",
        excluded.status.success() && exclude_excluded.contains("/.agents/\n"),
        format!(
            "contem /.agents/: {}",
            exclude_excluded.contains("/.agents/\n")
        ),
    );
    let back = ctx.ktu(&dir, &["init", "--git-tracked"], "")?;
    gate.check(
        "--git-tracked reverte ao estado inicial",
        back.status.success() && read(&dir.join(".git/info/exclude"))? == exclude,
        "exclude reposto ao estado inicial".to_owned(),
    );
    // Precedencia: flag > params > local > global (observada por comportamento e por config).
    write(
        &dir.join("xdg/ktu/config.toml"),
        "model = \"global\"\nprovider = \"llama\"\n",
    )?;
    write(
        &dir.join(".agents/config.toml"),
        "model = \"local\"\nprovider = \"llama\"\n",
    )?;
    let effective = ctx.ktu_ok(&dir, &["config", "get", "model"], "")?;
    gate.check(
        "local ganha ao global (M6)",
        effective.trim() == "local",
        effective.trim().to_owned(),
    );
    let flag = ctx.ktu(
        &dir,
        &[
            "run",
            "ola",
            "--provider",
            "mock",
            "--session",
            "p",
            "--max-steps",
            "1",
        ],
        "",
    )?;
    gate.check(
        "flag ganha a config (provider mock corre sem modelo)",
        flag.status.success(),
        flag.status.to_string(),
    );
    let params = ctx.ktu(
        &dir,
        &[
            "run",
            "--params",
            "{\"prompt\":\"ola\",\"provider\":\"mock\",\"session\":\"q\",\"max_steps\":1}",
        ],
        "",
    )?;
    gate.check(
        "params ganham a config",
        params.status.success(),
        params.status.to_string(),
    );
    // M3: a chave de auth nao ecoa no log nem no contexto.
    let secret = "sk-segredo-que-nao-pode-ecoar";
    write(
        &dir.join("xdg/ktu/auth.toml"),
        &format!("[providers]\nllama = \"{secret}\"\n"),
    )?;
    let listed = ctx.ktu_ok(&dir, &["config", "list", "--json"], "")?;
    gate.check(
        "M3: a chave de auth nao ecoa (redigida)",
        !listed.contains(secret),
        format!("contem segredo: {}", listed.contains(secret)),
    );
    // M5: um guardrail global e piso aditivo (o projecto nao o desliga).
    let deny = "---\nid: deny-env\nversion: 1\nfact: path\nmatch: '*.env'\ndecision: deny\nreason: segredos\n---\n";
    let allow = "---\nid: allow-env\nversion: 1\nfact: path\nmatch: '*.env'\ndecision: allow\nreason: quero\n---\n";
    write(&dir.join("xdg/ktu/guardrails/deny-env.md"), deny)?;
    write(&dir.join(".agents/guardrails/allow-env.md"), allow)?;
    write(&dir.join("segredos.env"), "CHAVE=1\n")?;
    let script = dir.join("m5.jsonl");
    write(
        &script,
        "{\"text\":\"\",\"tool_calls\":[{\"id\":\"c1\",\"name\":\"read_file\",\"arguments\":\"{\\\"path\\\":\\\"segredos.env\\\"}\"}]}\n{\"text\":\"fim\"}\n",
    )?;
    let m5 = ctx.ktu(
        &dir,
        &[
            "run",
            "le segredos.env",
            "--provider",
            "scripted",
            "--script",
            &script.to_string_lossy(),
            "--session",
            "m5",
            "--max-steps",
            "2",
        ],
        "",
    )?;
    let log = read(&dir.join(".ktu/sessions/m5.jsonl")).unwrap_or_default();
    gate.check(
        "M5: guardrail global nao e desligavel pelo projecto",
        m5.status.success() && log.contains("\"allowed\":false"),
        format!(
            "exit={} allowed=false: {} ({})",
            m5.status,
            log.contains("\"allowed\":false"),
            String::from_utf8_lossy(&m5.stderr)
                .lines()
                .last()
                .unwrap_or_default()
        ),
    );
    // Diagnostico: o `doctor` do projecto le o marcador de versao do bloco de
    // protocolo (que ate agora era escrito e nunca lido) e aponta as outras portas.
    let healthy = ctx.ktu_ok(&dir, &["doctor", "--json"], "")?;
    gate.check(
        "o doctor do projecto esta em dia depois do init",
        healthy.contains("\"healthy\":true")
            && healthy.contains("ktu memory doctor")
            && healthy.contains("\"name\":\"config\""),
        format!("healthy={}", healthy.contains("\"healthy\":true")),
    );
    // O init nao gera conteudo sobre o ktu: nem skill `ktu`, nem bloco no AGENTS.md.
    let skill = dir.join(".agents/skills/ktu/SKILL.md");
    gate.check(
        "o init nao gera a skill do ktu (as convencoes vivem na base do prompt)",
        !skill.exists(),
        format!("{} existe: {}", skill.display(), skill.exists()),
    );
    // O `AGENTS.md` e do projecto: o `ktu init` nao escreve nele (ADR 0021).
    let human = "# AGENTS\n\nRegras humanas.\n";
    write(&dir.join("AGENTS.md"), human)?;
    let after = ctx.ktu(&dir, &["init"], "")?;
    gate.check(
        "o init nao toca no AGENTS.md do projecto",
        after.status.success() && read(&dir.join("AGENTS.md"))? == human,
        format!("intacto: {}", read(&dir.join("AGENTS.md"))? == human),
    );
    let memory = ctx.ktu_ok(&dir, &["memory", "doctor", "--json"], "")?;
    gate.check(
        "o diagnostico da memoria tem porta propria",
        memory.contains("\"command\":\"memory doctor\"") && memory.contains("\"healthy\":true"),
        format!(
            "command=memory doctor: {}",
            memory.contains("memory doctor")
        ),
    );

    gate.artifact("exclude.txt", exclude);
    gate.artifact("gitattributes.txt", attributes);
    gate.artifact("m5.jsonl", log);
    gate.artifact("doctor.json", healthy);
    gate.artifact("memory-doctor.json", memory);
    Ok(gate)
}

/// G15 — eixo de memória: escreve, recupera numa sessão nova, projeção derivada.
pub(crate) fn g15(ctx: &Ctx) -> Result<Gate, String> {
    let mut gate = Gate::new(
        "G15",
        "Eixo de memoria: escreve, recupera noutra sessao, derivado",
    );
    let dir = ctx.fresh("g15")?;
    write(
        &dir.join("a.txt"),
        "conteudo
",
    )?;
    // A escrita não depende do modelo: o gatilho `session_end` grava no fim do turno.
    let first = ctx.ktu(
        &dir,
        &[
            "run",
            "le a.txt",
            "--provider",
            "mock",
            "--session",
            "s1",
            "--max-steps",
            "4",
        ],
        "",
    )?;
    let first_log = String::from_utf8_lossy(&first.stderr).into_owned();
    gate.check(
        "sessao 1 grava memoria no fim do turno",
        first.status.success() && first_log.contains("memory.write"),
        format!(
            "exit={} memory.write={}",
            first.status,
            first_log.contains("memory.write")
        ),
    );
    let notes = read(&dir.join(".agents/memory/notes.jsonl"))?;
    gate.check(
        "notes.jsonl tem a nota com proveniencia do agente",
        count(&notes, "\"statement\"") >= 1 && notes.contains("\"agent\":\"ktu\""),
        format!("linhas={}", notes.lines().count()),
    );
    let listed = ctx.ktu_ok(&dir, &["memory", "list"], "")?;
    gate.check(
        "ktu memory list mostra a nota",
        listed.contains("conteudo"),
        format!("linhas={}", listed.lines().count()),
    );
    // Sessão nova: o arranque recorda a memória (prefixo do system, não o log).
    let second = ctx.ktu(
        &dir,
        &[
            "run",
            "conteudo",
            "--provider",
            "mock",
            "--session",
            "s2",
            "--max-steps",
            "4",
        ],
        "",
    )?;
    let second_log = String::from_utf8_lossy(&second.stderr).into_owned();
    gate.check(
        "sessao nova recupera a memoria (memory.recall)",
        second.status.success() && second_log.contains("memory.recall"),
        format!(
            "exit={} memory.recall={}",
            second.status,
            second_log.contains("memory.recall")
        ),
    );
    // Índice derivado: apagar `.idx` e repetir dá o mesmo resultado (I17).
    let export_before = ctx.ktu_ok(&dir, &["memory", "export"], "")?;
    let derived = dir.join(".agents/memory/.idx");
    if derived.exists() {
        std::fs::remove_dir_all(&derived).map_err(|error| format!("idx: {error}"))?;
    }
    let export_after = ctx.ktu_ok(&dir, &["memory", "export"], "")?;
    gate.check(
        "apagar o indice derivado nao perde nada (recall identico)",
        export_before == export_after,
        format!(
            "bytes antes={} depois={}",
            export_before.len(),
            export_after.len()
        ),
    );
    // Projeção do log em notas derivadas, com proveniência (fonte read-only).
    let notes_before = read(&dir.join(".agents/memory/notes.jsonl"))?;
    let projected = ctx.ktu(&dir, &["memory", "project"], "")?;
    let derived_notes = read(&dir.join(".agents/memory/session-log.jsonl"))?;
    gate.check(
        "a projecao do log escreve notas derivadas com proveniencia",
        projected.status.success() && derived_notes.contains("\"agent\":\"ktu\""),
        format!("linhas={}", derived_notes.lines().count()),
    );
    gate.check(
        "a projecao nao altera o registo (fonte read-only)",
        read(&dir.join(".agents/memory/notes.jsonl"))? == notes_before,
        "notes.jsonl inalterado".to_owned(),
    );
    gate.artifact("notes.jsonl", notes);
    gate.artifact("session-log.jsonl", derived_notes);
    gate.artifact("stderr-recall.txt", second_log);
    // Lado do knudge: `ask --agent` (proveniência) e a fonte JSONL read-only vivem
    // no workspace do knudge — o gate corre-os como evidência (o contrato é I17).
    let manifest = ctx.root.join("crates/knudge/Cargo.toml");
    let ai = exec(
        Path::new("cargo"),
        &[
            "test",
            "--manifest-path",
            &manifest.to_string_lossy(),
            "-p",
            "knudge-cli",
            "--test",
            "ai",
        ],
        &ctx.root,
        "",
    )?;
    gate.check(
        "knudge: `ask --agent` filtra por proveniencia (fonte JSONL)",
        ai.status.success(),
        format!("exit={} {}", ai.status, summary_line(&ai)),
    );
    let corpus = exec(
        Path::new("cargo"),
        &[
            "test",
            "--manifest-path",
            &manifest.to_string_lossy(),
            "-p",
            "knudge-core",
            "corpus",
        ],
        &ctx.root,
        "",
    )?;
    gate.check(
        "knudge: a fonte JSONL e read-only, deduplica e tolera linhas mas",
        corpus.status.success(),
        format!("exit={} {}", corpus.status, summary_line(&corpus)),
    );
    gate.artifact("knudge-ai.txt", stdout(&ai));
    gate.artifact("knudge-corpus.txt", stdout(&corpus));
    Ok(gate)
}
