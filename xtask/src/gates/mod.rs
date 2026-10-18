//! `xtask gates` — gates da Fase 2 (**G10–G15**): usabilidade em cadeias extensas.
//!
//! Cada gate corre o `ktu` num espaço de trabalho isolado, verifica os critérios de
//! aceitação de `wiki/implementation_phase2/GATES.md` e arquiva os artefactos em
//! `wiki/gates/artifacts/<gate>/` + um resumo máquina-legível (`summary.json`).
//!
//! Os gates são **determinísticos** sempre que possível: `G12`–`G15` usam o provider
//! `mock` (eco) ou `scripted` (guião JSONL) e não precisam de modelo; `G10`/`G11` usam
//! um guião que percorre uma tarefa real de código, pelo que provam a **mecânica** da
//! cadeia longa sem depender de inferência (o modelo real é uma observação opcional).

mod model;
mod paths;
mod project;
mod support;

use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

use model::RealModel;
use support::{Ctx, Gate};

/// Diretório padrão dos artefactos.
const DEFAULT_OUT: &str = "wiki/gates";
/// Diretório padrão do espaço de trabalho.
const DEFAULT_WORK: &str = ".ktu/gates";
/// Binário padrão.
const DEFAULT_KTU: &str = "target/debug/ktu";

/// Opções de `xtask gates`.
struct Options {
    only: Vec<String>,
    ktu: PathBuf,
    work: PathBuf,
    out: PathBuf,
    base_url: String,
    model: String,
    real_model: bool,
}

impl Options {
    /// Interpreta os argumentos (`--only`, `--ktu`, `--work`, `--out`, `--real-model`).
    fn parse(args: &[String]) -> Result<Self, String> {
        let mut options = Self {
            only: Vec::new(),
            ktu: PathBuf::from(DEFAULT_KTU),
            work: PathBuf::from(DEFAULT_WORK),
            out: PathBuf::from(DEFAULT_OUT),
            base_url: "http://127.0.0.1:8080".to_owned(),
            model: "local".to_owned(),
            real_model: false,
        };
        let mut raw = args.iter();
        while let Some(arg) = raw.next() {
            match arg.as_str() {
                "--only" => {
                    let value = raw.next().ok_or("--only exige uma lista (ex.: G12,G13)")?;
                    options.only = value.split(',').map(|key| key.trim().to_owned()).collect();
                }
                "--ktu" => {
                    options.ktu = PathBuf::from(raw.next().ok_or("--ktu exige um caminho")?);
                }
                "--work" => {
                    options.work = PathBuf::from(raw.next().ok_or("--work exige um caminho")?);
                }
                "--out" => {
                    options.out = PathBuf::from(raw.next().ok_or("--out exige um caminho")?);
                }
                "--base-url" => {
                    raw.next()
                        .ok_or("--base-url exige um URL")?
                        .clone_into(&mut options.base_url);
                }
                "--model" => {
                    raw.next()
                        .ok_or("--model exige um nome")?
                        .clone_into(&mut options.model);
                }
                "--real-model" => options.real_model = true,
                other => return Err(format!("argumento desconhecido: {other}")),
            }
        }
        Ok(options)
    }

    /// `true` se o gate foi pedido.
    fn wants(&self, id: &str) -> bool {
        self.only.is_empty() || self.only.iter().any(|key| key == id)
    }
}

/// Corre os gates pedidos e escreve os artefactos + o resumo.
///
/// # Errors
/// Devolve mensagem legível se as opções forem inválidas, o binário faltar ou um
/// gate falhar (depois de imprimir o quadro de resultados).
pub(crate) fn run(args: &[String]) -> Result<(), String> {
    let options = Options::parse(args)?;
    let root = std::env::current_dir().map_err(|error| format!("cwd: {error}"))?;
    let ktu = absolute(&root, &options.ktu);
    if !ktu.exists() {
        return Err(format!(
            "binario ausente: {} (corra `cargo build -p ktu-cli`)",
            ktu.display()
        ));
    }
    // Os gates correm o binário com `current_dir` no espaço de trabalho: os caminhos
    // têm de ser **absolutos**, senão o `ktu`/guião deixam de ser encontrados.
    let ctx = Ctx {
        ktu,
        root: root.clone(),
        work: absolute(&root, &options.work),
        base_url: options.base_url.clone(),
        model: options.model.clone(),
    };
    fs::create_dir_all(&ctx.work).map_err(|error| format!("{}: {error}", ctx.work.display()))?;
    let mut gates: Vec<Gate> = Vec::new();
    for id in ["G10", "G11", "G12", "G13", "G14", "G15"] {
        if !options.wants(id) {
            continue;
        }
        gates.push(run_one(id, &ctx, &options)?);
    }
    if gates.is_empty() {
        return Err("nenhum gate selecionado".to_owned());
    }
    let out = absolute(&root, &options.out);
    write_artifacts(&out, &gates)?;
    print_table(&gates)?;
    let failed: Vec<&str> = gates
        .iter()
        .filter(|gate| !gate.ok())
        .map(|gate| gate.id)
        .collect();
    if failed.is_empty() {
        return Ok(());
    }
    Err(format!("gates vermelhos: {}", failed.join(", ")))
}

/// Resolve um caminho relativo contra a raiz do repositório.
fn absolute(root: &std::path::Path, path: &std::path::Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        root.join(path)
    }
}

/// Corre um gate pelo identificador.
fn run_one(id: &str, ctx: &Ctx, options: &Options) -> Result<Gate, String> {
    match id {
        "G10" => model::g10(ctx),
        "G11" => model::g11(
            ctx,
            if options.real_model {
                RealModel::Run
            } else {
                RealModel::Skip
            },
        ),
        "G12" => paths::g12(ctx),
        "G13" => paths::g13(ctx),
        "G14" => project::g14(ctx),
        "G15" => project::g15(ctx),
        other => Err(format!("gate desconhecido: {other}")),
    }
}

/// Escreve os artefactos de cada gate e o resumo JSON.
fn write_artifacts(out: &std::path::Path, gates: &[Gate]) -> Result<(), String> {
    for gate in gates {
        let dir = out.join("artifacts").join(gate.id);
        // Limpa antes de escrever: os artefactos são a evidência **desta** corrida.
        if dir.exists() {
            fs::remove_dir_all(&dir).map_err(|error| format!("{}: {error}", dir.display()))?;
        }
        fs::create_dir_all(&dir).map_err(|error| format!("{}: {error}", dir.display()))?;
        for artifact in &gate.artifacts {
            fs::write(dir.join(&artifact.name), &artifact.content)
                .map_err(|error| format!("{}: {error}", artifact.name))?;
        }
    }
    let summary = summary_json(gates);
    fs::create_dir_all(out).map_err(|error| format!("{}: {error}", out.display()))?;
    fs::write(out.join("summary.json"), summary).map_err(|error| format!("summary.json: {error}"))
}

/// Resumo máquina-legível dos gates (ordem estável).
fn summary_json(gates: &[Gate]) -> String {
    let value = serde_json::json!({
        "gates": gates
            .iter()
            .map(|gate| {
                serde_json::json!({
                    "id": gate.id,
                    "title": gate.title,
                    "ok": gate.ok(),
                    "checks": gate
                        .checks
                        .iter()
                        .map(|check| serde_json::json!({
                            "name": check.name,
                            "ok": check.ok,
                            "detail": check.detail,
                        }))
                        .collect::<Vec<_>>(),
                    "notes": gate.notes,
                })
            })
            .collect::<Vec<_>>(),
    });
    serde_json::to_string_pretty(&value).unwrap_or_default()
}

/// Imprime o quadro de resultados em `stdout` (dados).
fn print_table(gates: &[Gate]) -> Result<(), String> {
    let mut stdout = io::stdout().lock();
    let mut write = |text: String| -> Result<(), String> {
        write!(stdout, "{text}").map_err(|error| error.to_string())
    };
    for gate in gates {
        write(format!(
            "{} {} — {} ({}/{})\n",
            if gate.ok() { "OK  " } else { "FALHA" },
            gate.id,
            gate.title,
            gate.checks.iter().filter(|check| check.ok).count(),
            gate.checks.len()
        ))?;
        for check in &gate.checks {
            write(format!(
                "   [{}] {}: {}\n",
                if check.ok { "x" } else { " " },
                check.name,
                check.detail
            ))?;
        }
        for note in &gate.notes {
            write(format!("   nota: {note}\n"))?;
        }
    }
    Ok(())
}
