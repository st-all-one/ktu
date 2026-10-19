//! Gate **G16** — *steering* durável e idempotente (M7, Fase 3 / F1).
//!
//! Prova, no **binário**: a admissão fica no log antes da entrega, uma admissão pendente
//! é entregue no turno seguinte (o mesmo caminho que recupera de um *crash*) e a entrega
//! acontece **uma só vez**, mesmo repetindo a corrida. A idempotência **por id** (o front
//! repete o pedido) é provada pelos testes do kernel e da porta, arquivados com o gate.

use super::support::{Ctx, Gate, count, integrity, read, summary_line};

/// Corre o gate.
pub(crate) fn g16(ctx: &Ctx) -> Result<Gate, String> {
    let mut gate = Gate::new("G16", "Steering duravel e idempotente (M7)");
    let dir = ctx.fresh("g16")?;
    let log_path = dir.join(".ktu/sessions/s.jsonl");
    let args = ["session", "s", "--provider", "mock", "--max-steps", "4"];

    // 1. Turno com uma linha escrita **durante** ele: admitida, ainda não entregue.
    let first = ctx.ktu(&dir, &args, "ola\nfaz antes X\n/quit\n")?;
    let log = read(&log_path)?;
    gate.check(
        "a linha e admitida no log (durante o turno)",
        first.status.success() && count(&log, "\"event\":\"steer_queued\"") == 1,
        format!(
            "exit={} steer_queued={}",
            first.status,
            count(&log, "\"event\":\"steer_queued\"")
        ),
    );
    gate.check(
        "a admissao vem antes da entrega (sem entrega no mesmo turno)",
        count(&log, "\"event\":\"steer\"") == 0,
        format!("steer={}", count(&log, "\"event\":\"steer\"")),
    );
    gate.check(
        "I1/I2 limpos depois do turno",
        integrity(&log) == "ok",
        integrity(&log),
    );

    // 2. Retomar a **mesma** sessão: a admissão pendente entra na primeira fronteira.
    let second = ctx.ktu(&dir, &args, "continua\n/quit\n")?;
    let log = read(&log_path)?;
    gate.check(
        "a admissao pendente e entregue no turno seguinte",
        second.status.success() && count(&log, "\"event\":\"steer\"") == 1,
        format!(
            "exit={} steer={}",
            second.status,
            count(&log, "\"event\":\"steer\"")
        ),
    );
    gate.check(
        "a mensagem entregue e a que foi admitida",
        log.contains("\"text\":\"faz antes X\""),
        "texto presente no log".to_owned(),
    );

    // 3. Terceira corrida: nada duplica (entregue **exactamente uma vez**).
    let third = ctx.ktu(&dir, &args, "outra vez\n/quit\n")?;
    let log = read(&log_path)?;
    gate.check(
        "entregue exactamente uma vez (3 corridas na mesma sessao)",
        third.status.success() && count(&log, "\"event\":\"steer\"") == 1,
        format!(
            "exit={} steer={}",
            third.status,
            count(&log, "\"event\":\"steer\"")
        ),
    );
    gate.check(
        "I1/I2 limpos no fim",
        integrity(&log) == "ok",
        integrity(&log),
    );

    // 4. Idempotencia **por id** (o front repete o pedido): testes do kernel e da porta.
    let kernel = ctx.cargo(&["test", "-q", "-p", "ktu-core", "--lib", "a_repeated_"])?;
    let port = ctx.cargo(&[
        "test",
        "-q",
        "-p",
        "ktu-cli",
        "--test",
        "serve_http",
        "http_steering_is_admitted_once_per_id",
    ])?;
    gate.check(
        "o mesmo id nao duplica a mensagem (kernel)",
        kernel.status.success() && summary_line(&kernel).contains("2 passed"),
        summary_line(&kernel),
    );
    gate.check(
        "o mesmo id nao duplica pela porta HTTP",
        port.status.success() && summary_line(&port).contains("1 passed"),
        summary_line(&port),
    );

    gate.artifact("s.jsonl", &log);
    gate.artifact(
        "m7-tests.txt",
        format!(
            "## kernel\n{}\n## porta http\n{}",
            super::support::stdout(&kernel),
            super::support::stdout(&port)
        ),
    );
    gate.note(
        "A admissao e gravada na primeira fronteira do loop (o kernel nao tem I/O; a gravacao \
         do log e do actor). A janela que a admissao duravel cobre e: admissao registada -> \
         processo morre antes da entrega -> retomada entrega a mensagem uma vez."
            .to_owned(),
    );
    Ok(gate)
}
