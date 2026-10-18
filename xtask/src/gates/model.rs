//! Gates que exercitam uma **cadeia longa** (**G10** e **G11**).
//!
//! Usam um guião (`.jsonl`) que percorre uma tarefa real de código: prova a mecânica
//! da cadeia (contexto, tools, correcção, fecho de turnos) **sem** depender de
//! inferência. O modelo real é uma observação opcional (`--real-model`).

use std::path::Path;

use super::support::{Ctx, Gate, count, exec, read, stdout, write};

/// G10 — cadeia longa numa mesma sessão (≥15 turnos) com artefacto correcto.
pub(crate) fn g10(ctx: &Ctx) -> Result<Gate, String> {
    let mut gate = Gate::new(
        "G10",
        "Cadeia longa numa sessao (>=15 turnos, artefacto correto)",
    );
    let dir = ctx.fresh("g10")?;
    let fixture = ctx.root.join("wiki/gates/fixtures");
    let project = fixture.join("g10-project");
    let alvo = dir.join("alvo");
    copy_tree(&project, &alvo)?;
    let batch = dir.join("batch.jsonl");
    let script = dir.join("script.jsonl");
    copy(&fixture.join("g10-batch.jsonl"), &batch)?;
    copy(&fixture.join("g10-script.jsonl"), &script)?;
    let turns = count(&read(&batch)?, "prompt");
    let chain = ctx.ktu(
        &alvo,
        &[
            "run",
            "--batch",
            &batch.to_string_lossy(),
            "--sequential",
            "--provider",
            "scripted",
            "--script",
            &script.to_string_lossy(),
            "--session",
            "g10",
            "--max-steps",
            "4",
        ],
        "",
    )?;
    gate.check(
        "a cadeia corre de ponta a ponta",
        chain.status.success(),
        format!(
            "exit={} {}",
            chain.status,
            String::from_utf8_lossy(&chain.stderr)
                .lines()
                .last()
                .unwrap_or_default()
        ),
    );
    let log = read(&alvo.join(".ktu/sessions/g10.jsonl"))?;
    let starts = count(&log, "\"event\":\"turn_start\"");
    gate.check(
        ">=15 turnos na mesma sessao",
        starts >= 15 && starts == turns,
        format!("turnos={starts} (lote={turns})"),
    );
    let ends = count(&log, "\"event\":\"turn_end\"");
    gate.check(
        "todos os turnos fecham com StopReason normal (I1/I5)",
        ends == starts && count(&log, "\"reason\":\"end_turn\"") == starts,
        format!(
            "turn_end={ends} end_turn={} (sem estagnacao nem erro)",
            count(&log, "\"reason\":\"end_turn\"")
        ),
    );
    gate.check(
        "o log nao tem compactacao (a instrucao inicial fica no contexto, I14)",
        count(&log, "\"event\":\"session_compact\"") == 0,
        format!(
            "compactacoes={}",
            count(&log, "\"event\":\"session_compact\"")
        ),
    );
    // O artefacto é o que interessa: o alvo passa os seus testes.
    let test = exec(Path::new("cargo"), &["test", "--quiet"], &alvo, "")?;
    gate.check(
        "o artefacto esta correto (`cargo test` verde no repo alvo)",
        test.status.success(),
        format!(
            "exit={} {}",
            test.status,
            stdout(&test).lines().last().unwrap_or_default()
        ),
    );
    let lib = read(&alvo.join("src/lib.rs"))?;
    gate.check(
        "a soma foi corrigida e o modulo media ligado",
        lib.contains("a + b") && lib.contains("pub mod media;"),
        "src/lib.rs contem `a + b` e `pub mod media;`".to_owned(),
    );
    let media = read(&alvo.join("src/media.rs")).unwrap_or_default();
    gate.check(
        "o modulo novo tem teste",
        media.contains("fn media_de_tres") && media.contains("fn media_de_lista_vazia"),
        format!("bytes={}", media.len()),
    );
    gate.artifact("g10.jsonl", log);
    gate.artifact("src-lib.rs", lib);
    gate.artifact("src-media.rs", media);
    gate.artifact("cargo-test.txt", stdout(&test));
    gate.artifact("script.jsonl", read(&script)?);
    gate.note(format!(
        "Leituras por ficheiro (nao-estagnacao): {}",
        reads(&alvo.join(".ktu/sessions/g10.jsonl"))?
    ));
    gate.note("O guiao e deterministico (provider `scripted`): o mesmo gate, duas vezes, da o mesmo log — I6. O modelo real (Ornith 9B) nao e corrido aqui: neste host CPU-only mediu ~8 min por completion (ver o relatorio).");
    Ok(gate)
}

/// Se o gate corre também uma observação com o modelo real.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum RealModel {
    /// Só o guião determinístico (default).
    Skip,
    /// Corre também o modelo real (observação, não critério).
    Run,
}

/// G11 — robustez multi-modelo: tool call declarada em **texto** + reparação.
pub(crate) fn g11(ctx: &Ctx, real_model: RealModel) -> Result<Gate, String> {
    let mut gate = Gate::new(
        "G11",
        "Robustez multi-modelo: tool call em texto e reparacao",
    );
    let dir = ctx.fresh("g11")?;
    let fixture = ctx.root.join("wiki/gates/fixtures");
    let project = fixture.join("g10-project");
    let alvo = dir.join("alvo");
    copy_tree(&project, &alvo)?;
    let script = fixture.join("g11-script.jsonl");
    let chain = ctx.ktu(
        &alvo,
        &[
            "run",
            "le o modulo",
            "--provider",
            "scripted",
            "--script",
            &script.to_string_lossy(),
            "--session",
            "g11",
            "--max-steps",
            "4",
        ],
        "",
    )?;
    let log = read(&alvo.join(".ktu/sessions/g11.jsonl"))?;
    gate.check(
        "uma tool call declarada em texto e executada",
        chain.status.success() && count(&log, "\"name\":\"read_file\"") >= 2,
        format!(
            "exit={} tool_result={}",
            chain.status,
            count(&log, "\"event\":\"tool_result\"")
        ),
    );
    gate.check(
        "o turno fecha com StopReason normal",
        count(&log, "\"reason\":\"end_turn\"") == 1,
        format!("end_turn={}", count(&log, "\"reason\":\"end_turn\"")),
    );
    gate.artifact("g11.jsonl", log);
    if real_model == RealModel::Skip {
        gate.note("Modelo real nao corrido (sem `--real-model`): a prova e o caminho declarado-em-texto, que e o que um modelo fraco produz.");
        return Ok(gate);
    }
    // Observação opcional: o mesmo prompt com o modelo real (não é critério de aceitação).
    let real = ctx.ktu(
        &alvo,
        &[
            "run",
            "le src/lib.rs e diz o que faz",
            "--provider",
            "llama",
            "--base-url",
            &ctx.base_url,
            "--model",
            &ctx.model,
            "--session",
            "g11-real",
            "--max-steps",
            "4",
        ],
        "",
    )?;
    let real_log = read(&alvo.join(".ktu/sessions/g11-real-model.jsonl")).unwrap_or_default();
    gate.artifact("g11-real-model.jsonl", &real_log);
    // **Observação**, não critério: um modelo real é não-determinístico (e este host
    // não corre o modelo de referência). O veredicto do gate fica no guião.
    gate.note(format!(
        "Observacao com o modelo real ({} em {}): exit={}, turnos fechados={}, resposta final={:?}",
        ctx.model,
        ctx.base_url,
        real.status,
        count(&real_log, "\"reason\":\"end_turn\""),
        stdout(&real).trim()
    ));
    Ok(gate)
}

/// Copia um ficheiro.
fn copy(from: &Path, to: &Path) -> Result<(), String> {
    let text = read(from)?;
    write(to, &text)
}

/// Copia uma árvore de ficheiros (sem `target`).
fn copy_tree(from: &Path, to: &Path) -> Result<(), String> {
    let entries =
        std::fs::read_dir(from).map_err(|error| format!("{}: {error}", from.display()))?;
    for entry in entries {
        let entry = entry.map_err(|error| format!("{}: {error}", from.display()))?;
        let name = entry.file_name();
        if name == "target" {
            continue;
        }
        let source = entry.path();
        let target = to.join(&name);
        if source.is_dir() {
            copy_tree(&source, &target)?;
        } else {
            copy(&source, &target)?;
        }
    }
    Ok(())
}

/// Contagem de leituras por ficheiro no log (relatório de não-estagnação).
fn reads(log: &Path) -> Result<String, String> {
    let text = read(log)?;
    let mut counts: std::collections::BTreeMap<String, usize> = std::collections::BTreeMap::new();
    for line in text.lines() {
        if !line.contains("\"event\":\"assistant\"") {
            continue;
        }
        for name in ["src/lib.rs", "src/media.rs", "README.md", "Cargo.toml"] {
            if line.contains(name) {
                let slot = counts.entry(name.to_owned()).or_default();
                *slot = slot.saturating_add(1);
            }
        }
    }
    Ok(counts
        .into_iter()
        .map(|(name, count)| format!("{name}={count}"))
        .collect::<Vec<_>>()
        .join(" "))
}
