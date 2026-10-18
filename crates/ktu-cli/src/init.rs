//! `ktu init` — prepara o ambiente do projecto, de forma **idempotente**.
//!
//! Cria o layout (`.agents/` conhecimento, `.ktu/` runtime), a skill do ktu, o
//! template de config, o bloco gerido no `AGENTS.md` e as exclusões/atributos do
//! git. Os transformadores de texto são **puros** ([`ktu_core::git`]); aqui só se
//! lê e escreve. `--force` reescreve os ficheiros **gerados** (skill, config) —
//! nunca a memória nem os guardrails editados pelo humano.

use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use ktu_core::git::{self, GitMode};
use ktu_core::{guardrails, layout};

use crate::output::{self, Format};
use crate::prime;

/// Política de reescrita dos ficheiros gerados (skill, config).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Overwrite {
    /// Só cria o que falta.
    Keep,
    /// Reescreve os ficheiros gerados (`--force`); nunca a memória nem os guardrails.
    Force,
}

/// Opções de `ktu init`.
pub(crate) struct Options {
    /// Política de reescrita.
    pub(crate) overwrite: Overwrite,
    /// Versionamento do conhecimento no git.
    pub(crate) git: GitMode,
    /// Imprime o protocolo no fim (salvo `--no-prompt`).
    pub(crate) prompt: bool,
}

/// Estado de um item gerido.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State {
    /// Não existia antes.
    New,
    /// Existia e foi actualizado.
    Updated,
    /// Já estava em dia.
    Unchanged,
}

impl State {
    const fn label(self) -> &'static str {
        match self {
            Self::New => "novo",
            Self::Updated => "atualizado",
            Self::Unchanged => "inalterado",
        }
    }
}

/// Relatório do `ktu init` (idempotente e auditável): `(nome, estado)` por item.
struct Report {
    root: PathBuf,
    in_repo: bool,
    items: Vec<(String, State)>,
}

impl Report {
    fn push(&mut self, name: impl Into<String>, state: State) {
        self.items.push((name.into(), state));
    }
}

/// Executa `ktu init` sob `root` e devolve o relatório.
fn run(root: &Path, options: &Options) -> io::Result<Report> {
    let mut report = Report {
        root: root.to_path_buf(),
        in_repo: false,
        items: Vec::new(),
    };

    // 1. Layout (conhecimento + runtime).
    for dir in layout::INIT_DIRS {
        let path = root.join(dir);
        let state = if path.is_dir() {
            State::Unchanged
        } else {
            fs::create_dir_all(&path)?;
            State::New
        };
        report.push(*dir, state);
    }

    // 2. Memória: fonte da verdade, criada vazia e nunca reescrita.
    let notes = root.join(layout::NOTES_FILE);
    report.push(layout::NOTES_FILE, ensure_file(&notes, &[])?);

    // 3. Guardrails (seed): só o que falta, nunca sobrescreve edições humanas.
    for (name, body) in guardrails::SEED {
        let path = root.join(".agents").join("guardrails").join(name);
        report.push(
            format!("guardrails/{name}"),
            ensure_file(&path, body.as_bytes())?,
        );
    }

    // 4. Skill e config (ficheiros gerados; `--force` reescreve).
    let skill = root.join(layout::SKILL_FILE);
    report.push(
        layout::SKILL_FILE,
        write_managed(&skill, layout::SKILL_BODY.as_bytes(), options.overwrite)?,
    );
    let config = root.join(layout::CONFIG_FILE);
    report.push(
        layout::CONFIG_FILE,
        write_managed(
            &config,
            layout::CONFIG_TEMPLATE.as_bytes(),
            options.overwrite,
        )?,
    );

    // 5. `AGENTS.md` (bloco gerido, idempotente).
    let agents = root.join(git::agents::FILE);
    report.push(
        git::agents::FILE,
        apply_block(&agents, git::agents::upsert)?,
    );

    // 6. Git (no-op fora de repositório).
    if let Some(git_dir) = git_dir(root) {
        report.in_repo = true;
        let exclude = git_dir.join("info").join("exclude");
        report.push(
            ".git/info/exclude",
            apply_block(&exclude, |text| git::exclude::apply(text, options.git))?,
        );
        let attributes = root.join(git::attributes::FILE);
        report.push(
            git::attributes::FILE,
            apply_block(&attributes, |text| {
                git::attributes::upsert(text, options.git)
            })?,
        );
    }

    Ok(report)
}

/// Lê um ficheiro como texto, tolerando UTF-8 inválido.
fn read_text(path: &Path) -> io::Result<String> {
    Ok(String::from_utf8_lossy_owned(fs::read(path)?))
}

/// Garante que `path` existe com `body` (nunca sobrescreve). Devolve o estado.
fn ensure_file(path: &Path, body: &[u8]) -> io::Result<State> {
    if path.is_file() {
        return Ok(State::Unchanged);
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, body)?;
    Ok(State::New)
}

/// Escreve `body` se ausente, ou reescreve-o com [`Overwrite::Force`].
fn write_managed(path: &Path, body: &[u8], overwrite: Overwrite) -> io::Result<State> {
    let existed = path.is_file();
    if existed && overwrite == Overwrite::Keep {
        return Ok(State::Unchanged);
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    if existed && fs::read(path).unwrap_or_default() == body {
        return Ok(State::Unchanged);
    }
    fs::write(path, body)?;
    Ok(if existed { State::Updated } else { State::New })
}

/// Aplica um transformador de texto gerido a `path`. Devolve o estado.
fn apply_block(path: &Path, render: impl Fn(&str) -> String) -> io::Result<State> {
    let existed = path.is_file();
    let original = if existed {
        read_text(path)?
    } else {
        String::new()
    };
    let updated = render(&original);
    if updated == original {
        return Ok(State::Unchanged);
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, updated.as_bytes())?;
    Ok(if existed { State::Updated } else { State::New })
}

/// Encontra o directório `.git` a partir de `root` (dir, ou ficheiro de *worktree*).
fn git_dir(root: &Path) -> Option<PathBuf> {
    let mut current = Some(root);
    while let Some(dir) = current {
        let dot_git = dir.join(".git");
        if dot_git.is_dir() {
            return Some(dot_git);
        }
        if dot_git.is_file() {
            let text = read_text(&dot_git).ok()?;
            let target = text.trim().strip_prefix("gitdir:")?.trim();
            let path = PathBuf::from(target);
            return Some(if path.is_absolute() {
                path
            } else {
                dir.join(path)
            });
        }
        current = dir.parent();
    }
    None
}

/// Texto humano do relatório.
fn render(report: &Report) -> String {
    let mut lines = vec![format!("ktu init — {}", report.root.display())];
    for (name, state) in &report.items {
        lines.push(format!("  {name} ({})", state.label()));
    }
    if report.in_repo {
        lines.push("  git: exclusões e atributos geridos".to_owned());
    } else {
        lines.push("  git: fora de repositório (no-op)".to_owned());
    }
    lines.push(String::new());
    lines.push("próximos: ktu prime   # protocolo do agente".to_owned());
    lines.join("\n")
}

/// Dados do relatório (envelope JSON).
fn data(report: &Report) -> serde_json::Value {
    let items: Vec<serde_json::Value> = report
        .items
        .iter()
        .map(|(name, state)| serde_json::json!({ "item": name, "state": state.label() }))
        .collect();
    serde_json::json!({ "root": report.root, "in_repo": report.in_repo, "items": items })
}

/// Executa `ktu init` no directório actual e emite o relatório.
pub(crate) fn run_command(options: &Options, format: Format) -> ExitCode {
    let Ok(root) = env::current_dir() else {
        output::emit_error(
            format,
            "init",
            "io",
            "nao foi possivel determinar o directório actual",
        );
        return ExitCode::FAILURE;
    };
    match run(&root, options) {
        Ok(report) => {
            let mut text = render(&report);
            if options.prompt {
                text.push('\n');
                text.push_str(prime::PROTOCOL.trim_end());
            }
            output::emit(format, "init", &text, &data(&report));
            ExitCode::SUCCESS
        }
        Err(error) => {
            output::emit_error(format, "init", "io", &error.to_string());
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use std::env;
    use std::fs;
    use std::io;
    use std::path::PathBuf;
    use std::process;

    use ktu_core::git::GitMode;
    use ktu_core::layout;

    use super::{Options, Overwrite, State, run};

    /// Raiz temporária isolada por teste e por processo.
    fn temp_root(tag: &str) -> PathBuf {
        let root = env::temp_dir().join(format!("ktu-init-{tag}-{}", process::id()));
        let _ignored = fs::remove_dir_all(&root);
        root
    }

    fn options(git: GitMode) -> Options {
        Options {
            overwrite: Overwrite::Keep,
            git,
            prompt: false,
        }
    }

    #[test]
    fn init_creates_layout_and_is_idempotent() -> io::Result<()> {
        let root = temp_root("layout");
        let report = run(&root, &options(GitMode::Tracked))?;
        assert!(root.join(layout::NOTES_FILE).is_file());
        assert!(root.join(layout::SKILL_FILE).is_file());
        assert!(root.join(layout::CONFIG_FILE).is_file());
        assert!(root.join("AGENTS.md").is_file());
        assert!(!report.in_repo);

        let again = run(&root, &options(GitMode::Tracked))?;
        assert!(
            again
                .items
                .iter()
                .all(|(_, state)| *state == State::Unchanged),
            "segunda execucao deve ser inalterada"
        );

        let _ignored = fs::remove_dir_all(&root);
        Ok(())
    }

    #[test]
    fn agents_block_is_upserted_once() -> io::Result<()> {
        let root = temp_root("agents");
        fs::create_dir_all(&root)?;
        fs::write(root.join("AGENTS.md"), "# AGENTS\n\nRegras humanas.\n")?;
        run(&root, &options(GitMode::Tracked))?;
        let text = fs::read_to_string(root.join("AGENTS.md"))?;
        assert!(text.contains("Regras humanas."));
        assert_eq!(text.matches("<!-- ktu:start -->").count(), 1);
        let _ignored = fs::remove_dir_all(&root);
        Ok(())
    }

    #[test]
    fn git_exclude_switches_modes_and_reverts() -> io::Result<()> {
        let root = temp_root("git");
        fs::create_dir_all(root.join(".git"))?;
        run(&root, &options(GitMode::Tracked))?;

        let exclude = root.join(".git/info/exclude");
        let tracked = fs::read_to_string(&exclude)?;
        assert!(tracked.contains("/.agents/memory/index/"));
        assert!(!tracked.contains("/.agents/\n"));

        run(&root, &options(GitMode::Excluded))?;
        let excluded = fs::read_to_string(&exclude)?;
        assert!(excluded.contains("/.agents/"));
        assert!(!excluded.contains("/.agents/memory/index/"));

        run(&root, &options(GitMode::Tracked))?;
        assert_eq!(fs::read_to_string(&exclude)?, tracked);
        assert!(
            fs::read_to_string(root.join(".gitattributes"))?.contains("# ktu:start"),
            "atributos geridos devem voltar ao modo tracked"
        );

        let _ignored = fs::remove_dir_all(&root);
        Ok(())
    }

    #[test]
    fn force_preserves_memory_and_human_guardrails() -> io::Result<()> {
        let root = temp_root("force");
        run(&root, &options(GitMode::Tracked))?;

        let notes = root.join(layout::NOTES_FILE);
        fs::write(&notes, b"{\"id\":\"x\"}\n")?;
        let guardrail = root.join(".agents/guardrails/deny-dotenv.md");
        fs::write(&guardrail, b"editado a mao\n")?;

        let forced = Options {
            overwrite: Overwrite::Force,
            git: GitMode::Tracked,
            prompt: false,
        };
        run(&root, &forced)?;

        assert_eq!(fs::read(&notes)?, b"{\"id\":\"x\"}\n");
        assert_eq!(fs::read(&guardrail)?, b"editado a mao\n");
        assert_eq!(
            fs::read_to_string(root.join(layout::SKILL_FILE))?,
            layout::SKILL_BODY
        );

        let _ignored = fs::remove_dir_all(&root);
        Ok(())
    }
}
