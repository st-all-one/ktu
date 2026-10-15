//! Execução da bateria: corre o `ktu`, arquiva artefactos e sonda a persistência.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

use super::artifact::{self, Conditions, Outcome};
use super::options::Options;
use super::plan::{Question, Step};

/// Corre cada passo, arquiva o artefacto e escreve o resumo.
pub(super) fn run_battery(
    out: &mut dyn Write,
    options: &Options,
    questions: &[Question],
    steps: &[Step],
) -> Result<(), String> {
    let conditions = Conditions {
        provider: &options.provider,
        model: &options.model,
        effort: &options.effort,
        agent: &options.subject,
    };
    let mut rows = Vec::new();
    for (index, step) in steps.iter().enumerate() {
        let Some(question) = questions.get(index) else {
            continue;
        };
        writeln!(out, "\n== {} ({}) ==", question.id, step.key)
            .map_err(|error| io_message(&error))?;
        let prompt = adapt_prompt(&question.issue, &options.subject);
        let answer = run_step(options, step, &prompt)?;
        let compacted = checkpoint_compacted(options, &step.session);
        let probe = if step.probe {
            Some(persistence_probe(options)?)
        } else {
            None
        };
        let chars = answer.chars().count();
        rows.push(Row {
            key: step.key.clone(),
            compacted,
            probe,
            chars,
        });
        let outcome = Outcome { compacted, probe };
        write_artifact(options, question, &conditions, &answer, outcome)?;
        writeln!(
            out,
            "   compacted={compacted} probe={probe:?} chars={chars}"
        )
        .map_err(|error| io_message(&error))?;
    }
    write_summary(options, &rows)
}

/// Imprime o plano da bateria sem correr o modelo.
pub(super) fn write_plan(
    out: &mut dyn Write,
    options: &Options,
    steps: &[Step],
) -> Result<(), String> {
    writeln!(out, "bateria de qualidade — sujeito {}", options.subject)
        .map_err(|error| io_message(&error))?;
    for step in steps {
        writeln!(
            out,
            "  {} sessao={} resume={} compact={} probe={}",
            step.key,
            step.session,
            step.chain.resumes(),
            step.chain.compacts(),
            step.probe
        )
        .map_err(|error| io_message(&error))?;
    }
    Ok(())
}

/// Corre um turno do sujeito e devolve o `stdout` (resposta final).
fn run_step(options: &Options, step: &Step, prompt: &str) -> Result<String, String> {
    let output = ktu_command(options, step, prompt)
        .output()
        .map_err(|error| format!("falha ao executar ktu: {error}"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("ktu {} falhou: {}", step.key, stderr.trim()));
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

/// Constrói o comando `ktu run` de um passo.
fn ktu_command(options: &Options, step: &Step, prompt: &str) -> Command {
    let mut command = Command::new(&options.ktu);
    command
        .arg("run")
        .arg("--root")
        .arg(&options.root)
        .arg("--provider")
        .arg(&options.provider)
        .arg("--base-url")
        .arg(&options.base_url)
        .arg("--model")
        .arg(&options.model)
        .arg("--effort")
        .arg(&options.effort)
        .arg("--max-steps")
        .arg(&options.max_steps)
        .arg("--session")
        .arg(&step.session);
    if step.chain.resumes() {
        command.arg("--resume");
    }
    command.arg("--").arg(prompt);
    command
}

/// Lê `compacted` do checkpoint da sessão (falso se ausente/ilegível).
fn checkpoint_compacted(options: &Options, session: &str) -> bool {
    let path = options
        .root
        .join(".ktu")
        .join("sessions")
        .join(format!("{session}.checkpoint.json"));
    let Ok(text) = fs::read_to_string(&path) else {
        return false;
    };
    let Ok(value) = serde_json::from_str::<serde_json::Value>(&text) else {
        return false;
    };
    value
        .get("compacted")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false)
}

/// Sonda de persistência: um processo novo (`memory list`) recupera o registo.
fn persistence_probe(options: &Options) -> Result<bool, String> {
    let output = Command::new(&options.ktu)
        .arg("memory")
        .arg("list")
        .arg("--root")
        .arg(&options.root)
        .output()
        .map_err(|error| format!("falha ao sondar memoria: {error}"))?;
    let text = String::from_utf8_lossy(&output.stdout);
    Ok(text.lines().any(|line| !line.trim().is_empty()))
}

/// Escreve o artefacto `pN/<subject>-pN.md`.
fn write_artifact(
    options: &Options,
    question: &Question,
    conditions: &Conditions<'_>,
    answer: &str,
    outcome: Outcome,
) -> Result<(), String> {
    let dir = options.output.join(&question.key);
    fs::create_dir_all(&dir).map_err(|error| format!("{}: {error}", dir.display()))?;
    let path = dir.join(format!("{}-{}.md", options.subject, question.key));
    let text = artifact::render(question, conditions, answer, outcome);
    fs::write(&path, text).map_err(|error| format!("{}: {error}", path.display()))
}

/// Escreve o resumo `wiki/quality_test/<subject>-summary.md`.
fn write_summary(options: &Options, rows: &[Row]) -> Result<(), String> {
    let path = options
        .output
        .join(format!("{}-summary.md", options.subject));
    let mut text = String::from("# Bateria de qualidade — sujeito ");
    text.push_str(&options.subject);
    text.push_str("\n\n| questao | compactado | sonda | chars |\n|---|---|---|---|\n");
    for row in rows {
        let key = &row.key;
        let compacted = if row.compacted { "true" } else { "false" };
        let probe = row
            .probe
            .map_or("—", |value| if value { "true" } else { "false" });
        let chars = row.chars;
        let line = format!("| {key} | {compacted} | {probe} | {chars} |\n");
        text.push_str(&line);
    }
    fs::write(&path, text).map_err(|error| format!("{}: {error}", path.display()))
}

/// Reescreve os paths absolutos dos referenciais para paths relativos à raiz.
fn adapt_prompt(issue: &str, subject: &str) -> String {
    issue
        .replace("[agent]", subject)
        .replace("/REF/", "REF/")
        .replace("/exploration/", "exploration/")
}

/// Mapeia um erro de I/O de `stdout`/`stderr` para mensagem.
fn io_message(error: &std::io::Error) -> String {
    error.to_string()
}

/// Uma linha do resumo da bateria.
struct Row {
    /// Chave da questão.
    key: String,
    /// Se a compactação real foi observada no checkpoint.
    compacted: bool,
    /// Resultado da sonda de persistência (só no último passo).
    probe: Option<bool>,
    /// Tamanho da resposta em caracteres.
    chars: usize,
}

/// Guarda a memória do projeto e deixa-a limpa durante a bateria.
pub(super) struct MemoryBackup {
    /// Caminho de `notes.jsonl`.
    path: PathBuf,
    /// Conteúdo original, se existia.
    content: Option<String>,
}

impl MemoryBackup {
    /// Guarda o conteúdo atual e esvazia `notes.jsonl`.
    pub(super) fn new(root: &Path) -> Result<Self, String> {
        let path = root.join(".agents").join("memory").join("notes.jsonl");
        let content = if path.is_file() {
            Some(
                fs::read_to_string(&path)
                    .map_err(|error| format!("{}: {error}", path.display()))?,
            )
        } else {
            None
        };
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|error| format!("{}: {error}", parent.display()))?;
        }
        fs::write(&path, b"").map_err(|error| format!("{}: {error}", path.display()))?;
        Ok(Self { path, content })
    }

    /// Restaura o conteúdo original (ou remove o ficheiro que criou).
    pub(super) fn restore(self) -> Result<(), String> {
        match self.content {
            Some(text) => fs::write(&self.path, text)
                .map_err(|error| format!("{}: {error}", self.path.display())),
            None => fs::remove_file(&self.path)
                .map_err(|error| format!("{}: {error}", self.path.display())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::adapt_prompt;

    #[test]
    fn adapt_prompt_relativizes_paths() {
        let text = "resumo de /REF/knudge em /exploration/knudge/[agent]/knudge.md";
        let adapted = adapt_prompt(text, "ktu");
        assert_eq!(
            adapted,
            "resumo de REF/knudge em exploration/knudge/ktu/knudge.md"
        );
    }
}
