//! Gate **G17** — REPL interactivo (Fase 3 / F2).
//!
//! Prova, no **binário**: os `/` que faltavam delegam nos verbos existentes (`/compact`
//! grava o mesmo corte que `--compact`, `/fork` copia, `/memory` lista, `/resume` muda de
//! sessão) e o Ctrl-C **cancela** o turno sem matar o processo. O caminho *pipe* continua
//! linha-a-linha (o G12 trava a regressão).

use std::io::Write as _;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::Duration;

use super::support::{Ctx, Gate, count, exec, integrity, read, stdout, write};

/// Corre o gate.
pub(crate) fn g17(ctx: &Ctx) -> Result<Gate, String> {
    let mut gate = Gate::new("G17", "REPL interactivo (comandos / e Ctrl-C)");
    let dir = ctx.fresh("g17")?;
    let args = ["session", "s", "--provider", "mock", "--max-steps", "4"];
    let log_path = dir.join(".ktu/sessions/s.jsonl");

    // Um turno para haver log (os comandos de gestão precisam de uma sessão persistida).
    let seeded = ctx.ktu(
        &dir,
        &[
            "run",
            "ola",
            "--provider",
            "mock",
            "--max-steps",
            "1",
            "--session",
            "s",
        ],
        "",
    )?;
    gate.check(
        "sessao semeada com um turno",
        seeded.status.success(),
        format!("exit={}", seeded.status),
    );

    // 1. `/help` lista os comandos novos.
    let help = ctx.ktu(&dir, &args, "/help\n/quit\n")?;
    let help_text = String::from_utf8_lossy(&help.stderr).into_owned();
    gate.check(
        "o /help lista /compact, /fork, /memory e /resume",
        ["/compact", "/fork", "/memory", "/resume"]
            .iter()
            .all(|command| help_text.contains(command)),
        "ajuda do REPL".to_owned(),
    );

    // 2. `/memory` delega no `ktu memory list`.
    write(
        &dir.join(".agents/memory/notes.jsonl"),
        "{\"body\":\"corpo\",\"frontmatter\":{\"body_hash\":\"abc\",\"created_at\":\"2026-10-18T00:00:00.000Z\",\"id\":\"fact_abcabcab\",\"provenance\":{\"activity\":\"write\",\"agent\":\"ktu\"},\"schema_version\":2,\"statement\":\"a memoria funciona\",\"tags\":[\"teste\"],\"type\":\"fact\"}}\n",
    )?;
    let memory = ctx.ktu(&dir, &args, "/memory\n/quit\n")?;
    let memory_text = String::from_utf8_lossy(&memory.stderr).into_owned();
    gate.check(
        "o /memory mostra as notas do projecto",
        memory_text.contains("fact_abcabcab") && memory_text.contains("a memoria funciona"),
        "nota listada no REPL".to_owned(),
    );

    // 3. `/fork` copia o log byte-a-byte.
    let forked = ctx.ktu(&dir, &args, "/fork copia\n/quit\n")?;
    let source = read(&log_path)?;
    let copied = read(&dir.join(".ktu/sessions/copia.jsonl"))?;
    gate.check(
        "o /fork copia a sessao (byte-a-byte)",
        forked.status.success() && copied == source,
        format!("exit={} igual={}", forked.status, copied == source),
    );

    // 4. `/resume` muda de sessão: o turno seguinte escreve no log da sessão nova.
    let switched = ctx.ktu(&dir, &args, "/resume copia\nola de novo\n/quit\n")?;
    let copied = read(&dir.join(".ktu/sessions/copia.jsonl"))?;
    gate.check(
        "o /resume muda de sessao",
        switched.status.success() && copied.contains("ola de novo"),
        format!("exit={}", switched.status),
    );

    // 5. `/compact` e `--compact` produzem o **mesmo** corte (delegação, não reimplementação).
    let mut keep_from: Vec<String> = Vec::new();
    for session in ["c1", "c2"] {
        for prompt in ["um", "dois", "tres"] {
            let first = !dir.join(format!(".ktu/sessions/{session}.jsonl")).exists();
            let mut turn = vec![
                "run",
                prompt,
                "--provider",
                "mock",
                "--max-steps",
                "1",
                "--session",
                session,
            ];
            if !first {
                turn.push("--resume");
            }
            let _ignored = ctx.ktu(&dir, &turn, "")?;
        }
    }
    let _ignored = ctx.ktu(
        &dir,
        &[
            "run",
            "quatro",
            "--provider",
            "mock",
            "--max-steps",
            "1",
            "--session",
            "c1",
            "--resume",
            "--compact",
        ],
        "",
    )?;
    keep_from.push(cut_of(&read(
        &dir.join(".ktu/sessions/c1.checkpoint.json"),
    )?)?);
    let compacted = ctx.ktu(
        &dir,
        &["session", "c2", "--provider", "mock", "--max-steps", "1"],
        "/compact\n/quit\n",
    )?;
    keep_from.push(cut_of(&read(
        &dir.join(".ktu/sessions/c2.checkpoint.json"),
    )?)?);
    let same_cut = keep_from
        .first()
        .zip(keep_from.get(1))
        .is_some_and(|(flag, repl)| flag == repl && flag != "null");
    gate.check(
        "o /compact grava o mesmo corte que --compact",
        compacted.status.success() && same_cut,
        format!("repl_exit={} iguais={same_cut}", compacted.status),
    );
    // O log cru fica intacto (a compactação é uma projeção).
    let log = read(&dir.join(".ktu/sessions/c2.jsonl"))?;
    gate.check(
        "o /compact nao altera o log cru",
        count(&log, "\"event\":\"turn_start\"") == 3,
        format!("turnos={}", count(&log, "\"event\":\"turn_start\"")),
    );

    // 6. Ctrl-C durante um turno: o processo **não** morre e o turno fecha `Cancelled`.
    let (exit_ok, cancelled) = ctrl_c(ctx, &dir)?;
    gate.check(
        "Ctrl-C cancela o turno sem matar o processo",
        exit_ok,
        "o processo saiu com sucesso".to_owned(),
    );
    gate.check(
        "o turno cancelado fecha com StopReason::Cancelled",
        cancelled,
        "reason=cancelled no log".to_owned(),
    );
    let log = read(&log_path)?;
    gate.check(
        "I1/I2 limpos depois do Ctrl-C",
        integrity(&log) == "ok",
        integrity(&log),
    );

    gate.artifact("s.jsonl", &log);
    gate.artifact(
        "c2.checkpoint.json",
        read(&dir.join(".ktu/sessions/c2.checkpoint.json"))?,
    );
    gate.artifact("help.txt", help_text);
    gate.note(
        "A Tab (completar / e caminhos) continua **adiada**: exige *raw mode* e gestão do \
         terminal em volta dos logs do turno (o REPL escreve em stderr enquanto o utilizador \
         escreve). O resto do F2 esta entregue."
            .to_owned(),
    );
    Ok(gate)
}

/// Corte de compactação (`keep_from`) de um checkpoint, como texto estável.
fn cut_of(checkpoint: &str) -> Result<String, String> {
    let value: serde_json::Value = serde_json::from_str(checkpoint)
        .map_err(|error| format!("checkpoint invalido: {error}"))?;
    Ok(value
        .get("keep_from")
        .map_or_else(|| "null".to_owned(), ToString::to_string))
}

/// Corre o REPL com um plugin lento e envia SIGINT a meio do turno.
///
/// Devolve `(processo_saiu_bem, turno_cancelado)`.
fn ctrl_c(ctx: &Ctx, dir: &Path) -> Result<(bool, bool), String> {
    write(
        &dir.join(".ktu/plugins/slow/plugin.toml"),
        "name = \"slow\"\nentrypoint = \"./run.sh\"\nevents = [\"model.request\"]\nmode = \"observe\"\ntimeout_ms = 6000\n",
    )?;
    write(
        &dir.join(".ktu/plugins/slow/run.sh"),
        "#!/bin/sh\nsleep 2\necho '{\"verdict\":\"pass\"}'\n",
    )?;
    let script = dir.join(".ktu/plugins/slow/run.sh");
    let chmod = exec(
        Path::new("chmod"),
        &["+x", &script.to_string_lossy()],
        dir,
        "",
    )?;
    if !chmod.status.success() {
        return Err(format!("chmod falhou: {}", stdout(&chmod)));
    }

    let mut child = Command::new(&ctx.ktu)
        .args(["session", "s", "--provider", "mock", "--max-steps", "4"])
        .current_dir(dir)
        .env("XDG_CONFIG_HOME", dir.join("xdg"))
        .env("HOME", dir.join("home"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("falha ao executar o ktu: {error}"))?;
    if let Some(stdin) = child.stdin.as_mut() {
        stdin
            .write_all(b"ola\n")
            .map_err(|error| format!("stdin: {error}"))?;
    }
    std::thread::sleep(Duration::from_millis(1000));
    let killed = exec(
        Path::new("kill"),
        &["-INT", &child.id().to_string()],
        dir,
        "",
    )?;
    if !killed.status.success() {
        return Err("nao foi possivel enviar SIGINT".to_owned());
    }
    if let Some(stdin) = child.stdin.as_mut() {
        let _ignored = stdin.write_all(b"/quit\n");
    }
    drop(child.stdin.take());
    let output = child
        .wait_with_output()
        .map_err(|error| format!("espera: {error}"))?;
    let log = read(&dir.join(".ktu/sessions/s.jsonl"))?;
    Ok((
        output.status.success(),
        log.contains("\"reason\":\"cancelled\""),
    ))
}
