//! Gate **G22** — travões (links da wiki, `cargo doc` e knudge na CI) — Fase 3 / F7.
//!
//! Fecha as lacunas **L3** (nenhum gate olhava para a documentação) e **L1** (o
//! `crates/knudge` é workspace próprio e ficava fora de todos os travões):
//!
//! 1. `check-links` falha num link **deliberadamente quebrado**, com o par
//!    (ficheiro, alvo), e passa na árvore real;
//! 2. uma âncora inexistente é **aviso**, não erro (o ficheiro existe);
//! 3. `cargo doc` com `RUSTDOCFLAGS="-D warnings"` não produz avisos (L3);
//! 4. o portão do knudge existe (`make check-knudge`) e a CI chama-o (L1);
//! 5. a suite do knudge **passa** (o mesmo comando do passo da CI).

use super::support::{Ctx, Gate, exec, stdout};

/// Caminho do workflow da CI (a prova de que o passo do knudge existe).
const CI: &str = ".github/workflows/ci.yml";

/// Corre o `check-links` sob um `root` alternativo e devolve (sucesso, saída).
fn links(ctx: &Ctx, root: &std::path::Path) -> Result<(bool, String), String> {
    let output = exec(
        std::path::Path::new("cargo"),
        &[
            "run",
            "-q",
            "-p",
            "xtask",
            "--",
            "check-links",
            "--root",
            &root.to_string_lossy(),
        ],
        &ctx.root,
        "",
    )?;
    let mut text = stdout(&output);
    text.push_str(&String::from_utf8_lossy(&output.stderr));
    Ok((output.status.success(), text))
}

/// Corre o gate.
pub(crate) fn g22(ctx: &Ctx) -> Result<Gate, String> {
    let mut gate = Gate::new("G22", "Travoes (links, doc e knudge na CI)");
    let dir = ctx.fresh("g22")?;

    // ── 1. A árvore real passa; um link quebrado falha com o par ──────────────
    let (ok, real) = links(ctx, std::path::Path::new("."))?;
    gate.check(
        "a arvore actual nao tem links quebrados",
        ok && real.contains("links relativos"),
        real.lines().next().unwrap_or_default().to_owned(),
    );

    let tree = dir.join("ensaio");
    super::support::write(
        &tree.join("wiki/index.md"),
        "---\n\n# Indice\n\n- [existe](guia.md)\n- [quebrado](nao-existe.md)\n",
    )?;
    super::support::write(
        &tree.join("wiki/guia.md"),
        "# Guia\n\nver [indice](index.md).\n",
    )?;
    let (broken_ok, broken) = links(ctx, &tree)?;
    gate.check(
        "um link quebrado falha com o par (ficheiro, alvo)",
        !broken_ok && broken.contains("wiki/index.md → nao-existe.md"),
        broken
            .lines()
            .find(|line| line.contains("nao-existe.md"))
            .unwrap_or_default()
            .trim()
            .to_owned(),
    );

    // ── 2. Âncora inexistente: aviso (o ficheiro existe), não erro ────────────
    super::support::write(
        &tree.join("wiki/index.md"),
        "---\n\n# Indice\n\n- [guia](guia.md#seccao-que-nao-existe)\n",
    )?;
    let (anchor_ok, anchor) = links(ctx, &tree)?;
    gate.check(
        "ancora inexistente e aviso, nao erro",
        anchor_ok && anchor.contains("ancora `#seccao-que-nao-existe` nao encontrada"),
        anchor
            .lines()
            .find(|line| line.contains("ancora"))
            .unwrap_or_default()
            .trim()
            .to_owned(),
    );

    // ── 3. `cargo doc` sem avisos (links intra-doc) ──────────────────────────
    let doc = ctx.cargo_env(
        &["doc", "--workspace", "--no-deps"],
        &[("RUSTDOCFLAGS", "-D warnings")],
    )?;
    let doc_out = format!("{}{}", stdout(&doc), String::from_utf8_lossy(&doc.stderr));
    gate.check(
        "o doc nao tem avisos",
        doc.status.success() && !doc_out.contains("warning:"),
        format!(
            "{} (avisos: {})",
            doc.status,
            doc_out.matches("warning:").count()
        ),
    );

    // ── 4. O portão do knudge existe e a CI chama-o (L1) ─────────────────────
    let makefile = super::support::read(&ctx.root.join("Makefile"))?;
    let workflow = super::support::read(&ctx.root.join(CI))?;
    let step: Vec<&str> = workflow
        .lines()
        .filter(|line| line.contains("check-knudge"))
        .collect();
    gate.check(
        "a CI corre o portao do knudge",
        makefile.contains("check-knudge:")
            && makefile.contains("cd crates/knudge && $(MAKE) check")
            && !step.is_empty(),
        step.first()
            .map_or("passo ausente", |line| line.trim())
            .to_owned(),
    );

    // A suite do knudge **corre** (mesmo comando do passo da CI): a prova de L1 nao e
    // so o wiring — o portao dele esta verde.
    let knudge = ctx.cargo(&[
        "test",
        "-q",
        "--manifest-path",
        "crates/knudge/Cargo.toml",
        "--workspace",
    ])?;
    let summaries: Vec<String> = stdout(&knudge)
        .lines()
        .filter(|line| line.starts_with("test result"))
        .map(str::to_owned)
        .collect();
    let red: Vec<&String> = summaries
        .iter()
        .filter(|line| !line.contains("0 failed"))
        .collect();
    gate.check(
        "a suite do knudge passa",
        knudge.status.success() && !summaries.is_empty() && red.is_empty(),
        format!(
            "{} binarios de teste, {} com falhas",
            summaries.len(),
            red.len()
        ),
    );

    gate.artifact("links.txt", real);
    gate.artifact("links-broken.txt", broken);
    gate.artifact("links-anchor.txt", anchor);
    gate.artifact("doc.txt", doc_out);
    gate.artifact("ci-knudge.txt", format!("{}\n", step.join("\n")));
    gate.artifact("knudge-tests.txt", format!("{}\n", summaries.join("\n")));
    gate.note(
        "O knudge e um workspace proprio (`exclude` na raiz): o `make check` da raiz nao o ve, \
         pelo que o portao dele corre num alvo dedicado (`make check-knudge`) chamado pela CI. O \
         gate prova as duas metades: o wiring (alvo no `Makefile` + passo no workflow) e a suite \
         (corre aqui o mesmo comando do passo; medido: ~6 s com o alvo quente, ~45 s com alvo vazio)."
            .to_owned(),
    );
    Ok(gate)
}
