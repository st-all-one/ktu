//! Apoio comum dos gates: resultado, verificações e execução do `ktu`.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

/// Veredicto de uma verificação (enum: um `bool` solto não diz nada).
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Verdict {
    /// Passou.
    Pass,
    /// Falhou.
    Fail,
}

impl Verdict {
    /// `true` se passou.
    fn passes(self) -> bool {
        self == Self::Pass
    }
}

impl From<bool> for Verdict {
    fn from(ok: bool) -> Self {
        if ok { Self::Pass } else { Self::Fail }
    }
}

/// Uma verificação de um gate (nome curto + veredicto + detalhe observável).
pub(crate) struct Check {
    /// O que se verifica (critério do gate).
    pub(crate) name: &'static str,
    /// Veredicto.
    pub(crate) ok: bool,
    /// Detalhe observado (número, hash, mensagem).
    pub(crate) detail: String,
}

/// Um artefacto arquivado (nome do ficheiro + conteúdo).
pub(crate) struct Artifact {
    /// Nome do ficheiro dentro de `artifacts/<gate>/`.
    pub(crate) name: String,
    /// Conteúdo (texto).
    pub(crate) content: String,
}

/// Resultado de um gate.
pub(crate) struct Gate {
    /// Identificador (`G10`).
    pub(crate) id: &'static str,
    /// Título do gate.
    pub(crate) title: &'static str,
    /// Verificações (todas têm de passar).
    pub(crate) checks: Vec<Check>,
    /// Artefactos arquivados.
    pub(crate) artifacts: Vec<Artifact>,
    /// Observações (desvios, medições).
    pub(crate) notes: Vec<String>,
}

impl Gate {
    /// Cria o gate vazio.
    pub(crate) fn new(id: &'static str, title: &'static str) -> Self {
        Self {
            id,
            title,
            checks: Vec::new(),
            artifacts: Vec::new(),
            notes: Vec::new(),
        }
    }

    /// Regista uma verificação.
    pub(crate) fn check(
        &mut self,
        name: &'static str,
        ok: impl Into<Verdict>,
        detail: impl Into<String>,
    ) {
        self.checks.push(Check {
            name,
            ok: ok.into().passes(),
            detail: detail.into(),
        });
    }

    /// Regista um artefacto.
    pub(crate) fn artifact(&mut self, name: impl Into<String>, content: impl Into<String>) {
        self.artifacts.push(Artifact {
            name: name.into(),
            content: content.into(),
        });
    }

    /// Regista uma observação.
    pub(crate) fn note(&mut self, text: impl Into<String>) {
        self.notes.push(text.into());
    }

    /// `true` se todas as verificações passaram.
    pub(crate) fn ok(&self) -> bool {
        self.checks.iter().all(|check| check.ok)
    }
}

/// Contexto de execução dos gates (binário, espaço de trabalho, modelo).
pub(crate) struct Ctx {
    /// Caminho do binário `ktu`.
    pub(crate) ktu: PathBuf,
    /// Raiz do repositório (para `cargo run -p ktu-embed`, guiões e fixtures).
    pub(crate) root: PathBuf,
    /// Espaço de trabalho isolado (`.ktu/gates`).
    pub(crate) work: PathBuf,
    /// *Base URL* do modelo (gates que exigem inferência).
    pub(crate) base_url: String,
    /// Modelo (gates que exigem inferência).
    pub(crate) model: String,
}

impl Ctx {
    /// Cria um espaço de trabalho limpo e devolve o seu caminho.
    pub(crate) fn fresh(&self, name: &str) -> Result<PathBuf, String> {
        let dir = self.work.join(name);
        if dir.exists() {
            fs::remove_dir_all(&dir).map_err(|error| format!("{}: {error}", dir.display()))?;
        }
        fs::create_dir_all(&dir).map_err(|error| format!("{}: {error}", dir.display()))?;
        Ok(dir)
    }

    /// Corre `ktu <args>` em `dir`, com `stdin` dado e a config do utilizador isolada.
    pub(crate) fn ktu(&self, dir: &Path, args: &[&str], input: &str) -> Result<Output, String> {
        exec(&self.ktu, args, dir, input)
    }

    /// Corre `ktu <args>` e devolve `stdout` (exige sucesso).
    pub(crate) fn ktu_ok(&self, dir: &Path, args: &[&str], input: &str) -> Result<String, String> {
        let output = self.ktu(dir, args, input)?;
        if !output.status.success() {
            return Err(format!(
                "ktu {} falhou ({}): {}",
                args.join(" "),
                output.status,
                String::from_utf8_lossy(&output.stderr).trim()
            ));
        }
        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    }

    /// Corre `cargo <args>` na raiz do repositório.
    pub(crate) fn cargo(&self, args: &[&str]) -> Result<Output, String> {
        exec(Path::new("cargo"), args, &self.root, "")
    }

    /// Corre `cargo <args>` na raiz com variáveis de ambiente extra (ex.: `RUSTDOCFLAGS`).
    pub(crate) fn cargo_env(&self, args: &[&str], env: &[(&str, &str)]) -> Result<Output, String> {
        exec_env(Path::new("cargo"), args, &self.root, "", env)
    }
}

/// Corre um comando arbitrário em `dir`, com a config do utilizador isolada.
pub(crate) fn exec(
    program: &Path,
    args: &[&str],
    dir: &Path,
    input: &str,
) -> Result<Output, String> {
    exec_env(program, args, dir, input, &[])
}

/// Como `exec`, com variáveis de ambiente extra.
pub(crate) fn exec_env(
    program: &Path,
    args: &[&str],
    dir: &Path,
    input: &str,
    env: &[(&str, &str)],
) -> Result<Output, String> {
    use std::io::Write as _;
    let mut child = Command::new(program)
        .args(args)
        .current_dir(dir)
        .env("XDG_CONFIG_HOME", dir.join("xdg"))
        .env("HOME", dir.join("home"))
        .envs(env.iter().copied())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("falha ao executar {}: {error}", program.display()))?;
    if let Some(stdin) = child.stdin.take() {
        let mut stdin = stdin;
        stdin
            .write_all(input.as_bytes())
            .map_err(|error| format!("falha ao escrever stdin: {error}"))?;
    }
    child
        .wait_with_output()
        .map_err(|error| format!("falha ao esperar o processo: {error}"))
}

/// Lê um ficheiro como texto (erro legível).
pub(crate) fn read(path: &Path) -> Result<String, String> {
    fs::read_to_string(path).map_err(|error| format!("{}: {error}", path.display()))
}

/// Escreve um ficheiro (criando os directórios).
pub(crate) fn write(path: &Path, content: &str) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| format!("{}: {error}", parent.display()))?;
    }
    fs::write(path, content).map_err(|error| format!("{}: {error}", path.display()))
}

/// `stdout` de uma execução, como texto.
pub(crate) fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

/// Conta as ocorrências de `needle` em `text`.
pub(crate) fn count(text: &str, needle: &str) -> usize {
    text.matches(needle).count()
}

/// Linha de resumo (`test result: …`) da saída de um `cargo test`.
pub(crate) fn summary_line(output: &Output) -> String {
    stdout(output)
        .lines()
        .find(|line| line.starts_with("test result"))
        .unwrap_or_default()
        .to_owned()
}

/// Verifica I1/I2 num log JSONL: turnos fechados e tool calls com resultado.
pub(crate) fn integrity(log: &str) -> String {
    let mut open = 0i64;
    let mut calls: Vec<String> = Vec::new();
    let mut missing = 0usize;
    for line in log.lines() {
        if line.contains("\"event\":\"turn_start\"") {
            open = open.saturating_add(1);
        }
        if line.contains("\"event\":\"turn_end\"") {
            open = open.saturating_sub(1);
        }
        if line.contains("\"event\":\"assistant\"") {
            calls.extend(ids(line));
        }
        if line.contains("\"event\":\"tool_result\"") {
            for id in ids(line) {
                if let Some(position) = calls.iter().position(|known| *known == id) {
                    calls.remove(position);
                }
            }
        }
    }
    if open != 0 {
        return format!("turnos abertos: {open}");
    }
    if !calls.is_empty() {
        missing = calls.len();
    }
    if missing == 0 {
        return "ok".to_owned();
    }
    format!("tool calls sem resultado: {missing}")
}

/// Extrai os `id`/`tool_call_id` citados numa linha (o sufixo `id":"…"`).
pub(crate) fn ids(line: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut rest = line;
    while let Some(start) = rest.find("id\":\"") {
        let tail = rest.get(start.saturating_add(5)..).unwrap_or_default();
        let Some(end) = tail.find('"') else {
            break;
        };
        out.push(tail.get(..end).unwrap_or_default().to_owned());
        rest = tail.get(end..).unwrap_or_default();
    }
    out
}

/// Hash estável (FNV-1a de 64 bits) — só para comparar conteúdos no relatório.
pub(crate) fn hash(text: &str) -> String {
    let mut value: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in text.as_bytes() {
        value ^= u64::from(*byte);
        value = value.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("{value:016x}")
}
