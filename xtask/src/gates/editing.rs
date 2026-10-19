//! Gate **G19** — edição sob concorrência (Fase 3 / F4).
//!
//! Prova, no **binário** e nos testes das tools: mutações ao **mesmo** caminho não se
//! sobrepõem (o ficheiro final é um dos escritos, e nenhuma edição se perde), caminhos
//! distintos correm em paralelo, o remédio **da regra** acompanha o estrutural e uma
//! âncora errada sugere as linhas próximas.

use super::support::{Ctx, Gate, read, summary_line, write};

/// Regra `deny` com remédio próprio.
const RULE: &str = "---\nid: deny-read\nversion: 1\napplies_to: '*'\nfact: tool\nmatch: read_file\ndecision: deny\nreason: leitura sensivel\nremedy: pede autorizacao ao responsavel\n---\n";

/// Guião de um passo com uma chamada de tool.
fn script(tool: &str, arguments: &str) -> String {
    let call = serde_json::json!({
        "text": "",
        "tool_calls": [{ "id": "c1", "name": tool, "arguments": arguments }],
    });
    format!("{call}\n{{\"text\":\"fim\"}}\n")
}

/// Corre o gate.
pub(crate) fn g19(ctx: &Ctx) -> Result<Gate, String> {
    let mut gate = Gate::new("G19", "Edicao sob concorrencia (fila, remedio, ancoras)");
    let dir = ctx.fresh("g19")?;

    // ── 1. A fila por caminho, nos testes das tools (concorrência real) ────────
    let queue = ctx.cargo(&["test", "-q", "-p", "ktu-tools", "--test", "mutation"])?;
    gate.check(
        "o mesmo caminho serializa e nenhuma edicao se perde",
        queue.status.success() && summary_line(&queue).contains("3 passed"),
        summary_line(&queue),
    );
    // A fila e **do processo**: duas sessoes (duas T2) do mesmo hub partilham-na.
    let shared = ctx.cargo(&[
        "test",
        "-q",
        "-p",
        "ktu-cli",
        "--bin",
        "ktu",
        "the_hub_shares_one_mutation_queue",
    ])?;
    gate.check(
        "o processo partilha uma fila entre sessoes",
        shared.status.success() && summary_line(&shared).contains("1 passed"),
        summary_line(&shared),
    );
    let paths = ctx.cargo(&["test", "-q", "-p", "ktu-tools", "--lib", "mutation::"])?;
    gate.check(
        "caminhos distintos correm em paralelo",
        paths.status.success() && summary_line(&paths).contains("2 passed"),
        summary_line(&paths),
    );

    // ── 2. Remédio da regra + estrutural na mesma mensagem ────────────────────
    write(&dir.join(".agents/guardrails/deny-read.md"), RULE)?;
    let deny_script = dir.join("deny.jsonl");
    write(
        &deny_script,
        &script("read_file", "{\"path\":\"notas.txt\"}"),
    )?;
    write(&dir.join("notas.txt"), "conteudo\n")?;
    let denied = ctx.ktu(
        &dir,
        &[
            "run",
            "le notas.txt",
            "--provider",
            "scripted",
            "--script",
            &deny_script.to_string_lossy(),
            "--session",
            "deny",
            "--max-steps",
            "2",
        ],
        "",
    )?;
    let deny_log = read(&dir.join(".ktu/sessions/deny.jsonl")).unwrap_or_default();
    gate.check(
        "a recusa traz o remedio da regra",
        denied.status.success() && deny_log.contains("pede autorizacao ao responsavel"),
        "remedio da regra".to_owned(),
    );
    gate.check(
        "a recusa nao perde o remedio estrutural do motivo",
        deny_log.contains("respeite o guardrail"),
        "remedio estrutural".to_owned(),
    );

    // ── 3. Âncora errada ⇒ sugestão das linhas próximas, ficheiro intacto ─────
    let original = "# Titulo\n\nfn soma(a: u32) -> u32 {\n    a + 1\n}\n";
    write(&dir.join("alvo.txt"), original)?;
    let edit_script = dir.join("edit.jsonl");
    write(
        &edit_script,
        &script(
            "edit_file",
            "{\"path\":\"alvo.txt\",\"edits\":[{\"old\":\"fn somma(a: u32) -> u32 {\",\"new\":\"x\"}]}",
        ),
    )?;
    let edited = ctx.ktu(
        &dir,
        &[
            "run",
            "edita",
            "--provider",
            "scripted",
            "--script",
            &edit_script.to_string_lossy(),
            "--session",
            "edit",
            "--max-steps",
            "2",
        ],
        "",
    )?;
    let edit_log = read(&dir.join(".ktu/sessions/edit.jsonl")).unwrap_or_default();
    gate.check(
        "a ancora errada sugere a linha correcta",
        edited.status.success()
            && edit_log.contains("mais proximas")
            && edit_log.contains("3: fn soma(a: u32) -> u32 {"),
        "sugestao com numero de linha".to_owned(),
    );
    gate.check(
        "a edicao falhada nao altera o ficheiro",
        read(&dir.join("alvo.txt"))? == original,
        "ficheiro intacto".to_owned(),
    );

    // ── 4. O caminho real de escrita (artefacto do estado final) ─────────────
    let write_script = dir.join("write.jsonl");
    write(
        &write_script,
        &script(
            "write_file",
            "{\"path\":\"saida.txt\",\"content\":\"final\\n\"}",
        ),
    )?;
    let written = ctx.ktu(
        &dir,
        &[
            "run",
            "escreve",
            "--provider",
            "scripted",
            "--script",
            &write_script.to_string_lossy(),
            "--session",
            "write",
            "--max-steps",
            "2",
        ],
        "",
    )?;
    let final_text = read(&dir.join("saida.txt")).unwrap_or_default();
    gate.check(
        "a escrita pelo binario deixa o ficheiro final esperado",
        written.status.success() && final_text == "final\n",
        format!("{final_text:?}"),
    );

    gate.artifact(
        "queue-tests.txt",
        format!("{}\n{}", summary_line(&queue), summary_line(&paths)),
    );
    gate.artifact("remedy.txt", deny_log);
    gate.artifact("editing.log", edit_log);
    gate.artifact("final.txt", final_text);
    gate.note(
        "A fila e **do processo** (Arc no `Inner` do hub, uma por lote): o `ktu serve` corre \
         um actor por sessao e o `--batch --parallel` uma thread por worker, pelo que uma fila \
         por turno nao serializaria nada entre sessoes. O que continua **fora**: dois processos no \
         mesmo projecto (lock de ficheiro) e a escrita atomica (temp + rename) — ver \
         `wiki/pendencias.md`."
            .to_owned(),
    );
    Ok(gate)
}
