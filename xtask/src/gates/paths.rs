//! Gates determinísticos de **caminho** e **concorrência** (G12, G13).

use super::support::{Ctx, Gate, count, hash, integrity, read, stdout, write};

/// G12 — independência do caminho (M4): `run`, REPL e crate ⇒ **log byte-idêntico**.
pub(crate) fn g12(ctx: &Ctx) -> Result<Gate, String> {
    let mut gate = Gate::new("G12", "Independencia do caminho (M4): run, REPL e crate");
    let dir = ctx.fresh("g12")?;
    write(&dir.join("a.txt"), "ola mundo\n")?;
    let run = ctx.ktu(
        &dir,
        &["run", "le a.txt", "--provider", "mock", "--session", "a"],
        "",
    )?;
    let repl = ctx.ktu(
        &dir,
        &["session", "b", "--provider", "mock"],
        "le a.txt\n/quit\n",
    )?;
    let root = dir.to_string_lossy().into_owned();
    let crate_run = ctx.cargo(&[
        "run",
        "-q",
        "-p",
        "ktu-embed",
        "--",
        "--root",
        &root,
        "--session",
        "c",
        "le a.txt",
    ])?;
    gate.check(
        "os tres caminhos terminam com sucesso",
        run.status.success() && repl.status.success() && crate_run.status.success(),
        format!(
            "run={} repl={} crate={}",
            run.status, repl.status, crate_run.status
        ),
    );
    let logs: Vec<String> = ["a", "b", "c"]
        .iter()
        .map(|id| {
            read(
                &dir.join(".ktu")
                    .join("sessions")
                    .join(format!("{id}.jsonl")),
            )
        })
        .collect::<Result<_, _>>()?;
    let hashes: Vec<String> = logs.iter().map(|log| hash(log)).collect();
    let shown = ["run", "repl", "crate"]
        .iter()
        .zip(&hashes)
        .map(|(path, hash)| format!("{path}={hash}"))
        .collect::<Vec<_>>()
        .join(" ");
    let identical = logs.windows(2).all(|pair| pair.first() == pair.get(1));
    gate.check(
        "derive_messages(log) byte-identico nos tres caminhos",
        identical,
        format!("fnv64: {shown}"),
    );
    gate.check(
        "cada caminho fecha o turno com StopReason normal (I5)",
        logs.iter().all(|log| count(log, "\"turn_end\"") == 1),
        format!(
            "turn_end por log: {:?}",
            logs.iter()
                .map(|log| count(log, "\"turn_end\""))
                .collect::<Vec<_>>()
        ),
    );
    // Determinismo (I6): o mesmo caminho, outra vez, dá o mesmo log.
    let repeat = ctx.ktu(
        &dir,
        &["run", "le a.txt", "--provider", "mock", "--session", "d"],
        "",
    )?;
    let repeated = read(&dir.join(".ktu/sessions/d.jsonl")).unwrap_or_default();
    gate.check(
        "determinismo: o mesmo caminho duas vezes da o mesmo log (I6)",
        repeat.status.success() && logs.first().is_some_and(|first| *first == repeated),
        format!(
            "fnv64: run={} repetido={}",
            hashes.first().cloned().unwrap_or_default(),
            hash(&repeated)
        ),
    );
    gate.artifact("run.jsonl", logs.first().cloned().unwrap_or_default());
    gate.artifact("repl.jsonl", logs.get(1).cloned().unwrap_or_default());
    gate.artifact("crate.jsonl", logs.get(2).cloned().unwrap_or_default());
    gate.note("O caminho 'crate' e o exemplo `ktu-embed` (fachada); usa o mesmo provider de demonstracao e o mesmo host de plugins do CLI, o que torna o log literalmente identico.");
    Ok(gate)
}

/// G13 — pipeline concorrente (uso #2): paralelo ordenado, lock e integridade.
pub(crate) fn g13(ctx: &Ctx) -> Result<Gate, String> {
    let mut gate = Gate::new(
        "G13",
        "Pipeline concorrente: paralelo ordenado, lock e integridade",
    );
    let dir = ctx.fresh("g13")?;
    write(&dir.join("a.txt"), "conteudo\n")?;
    let items: String = (0..4)
        .map(|_| "{\"prompt\":\"le a.txt\"}\n".to_owned())
        .collect();
    write(&dir.join("batch.jsonl"), &items)?;
    let parallel = ctx.ktu(
        &dir,
        &[
            "run",
            "--batch",
            "batch.jsonl",
            "--provider",
            "mock",
            "--max-steps",
            "2",
        ],
        "",
    )?;
    gate.check(
        "batch paralelo termina com sucesso",
        parallel.status.success(),
        parallel.status.to_string(),
    );
    let indexes: Vec<String> = stdout(&parallel)
        .lines()
        .filter_map(|line| line.split(']').next().map(str::to_owned))
        .collect();
    gate.check(
        "stdout ordenado pelo indice de entrada",
        indexes == ["[0", "[1", "[2", "[3"],
        indexes.join(" "),
    );
    let mut logs = Vec::new();
    for index in 0..4 {
        logs.push(read(
            &dir.join(".ktu")
                .join("sessions")
                .join(format!("batch-{index}.jsonl")),
        )?);
    }
    gate.check(
        "cada item corre na sua sessao (sem corrupcao)",
        logs.iter().all(|log| count(log, "\"turn_start\"") == 1),
        format!(
            "turnos por log: {:?}",
            logs.iter()
                .map(|log| count(log, "\"turn_start\""))
                .collect::<Vec<_>>()
        ),
    );
    let integrity: Vec<String> = logs.iter().map(|log| integrity(log)).collect();
    gate.check(
        "I1/I2 limpos em todos os logs",
        integrity.iter().all(|report| report == "ok"),
        integrity.join(" · "),
    );
    // Lock: um lock vivo recusa o segundo processo; largado, o turno corre.
    let sessions = dir.join(".ktu").join("sessions");
    write(&sessions.join("ocupada.lock"), "9999999999999\n")?;
    let blocked = ctx.ktu(
        &dir,
        &[
            "run",
            "ola",
            "--provider",
            "mock",
            "--session",
            "ocupada",
            "--max-steps",
            "1",
        ],
        "",
    )?;
    let stderr = String::from_utf8_lossy(&blocked.stderr).into_owned();
    gate.check(
        "lock vivo recusa o segundo processo com razao clara",
        !blocked.status.success() && stderr.contains("ocupada"),
        format!("exit={} stderr={}", blocked.status, stderr.trim()),
    );
    std::fs::remove_file(sessions.join("ocupada.lock"))
        .map_err(|error| format!("lock: {error}"))?;
    let free = ctx.ktu(
        &dir,
        &[
            "run",
            "ola",
            "--provider",
            "mock",
            "--session",
            "ocupada",
            "--max-steps",
            "1",
        ],
        "",
    )?;
    gate.check(
        "lock largado deixa o turno correr",
        free.status.success(),
        free.status.to_string(),
    );
    gate.artifact("batch-0.jsonl", logs.first().cloned().unwrap_or_default());
    gate.artifact("stderr-lock.txt", stderr);
    gate.note("O lock e por id de sessao (`.ktu/sessions/<id>.lock`), com `create_new` e expiracao a 30 s; nunca apagamos um lock alheio.");
    Ok(gate)
}
