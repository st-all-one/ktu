//! Montagem do system prompt (§2.11): base/directriz + `AGENTS.md` condensado.
//!
//! A composição é **pura** (texto entra, texto sai). A leitura dos ficheiros usa
//! a porta [`Fs`] e é **fail-open**: ausente ou ilegível = nada (nunca erro).

use std::path::Path;

use crate::ports::Fs;

/// Ficheiro que **substitui** a base do agente (`SYSTEM.md`).
///
/// Projecto: `.agents/SYSTEM.md` · utilizador: `<global>/SYSTEM.md`.
/// O de projecto tem precedência (homónimos não se combinam).
///
/// Nota: o nome é também usado pelo consumidor (a fachada) para compor o prompt
/// a partir do directório do projecto (`<root>/.agents/`).
pub const SYSTEM_FILE: &str = "SYSTEM.md";

/// Ficheiro que **acrescenta** ao fim do system prompt (`APPEND_SYSTEM.md`).
///
/// Projecto: `.agents/APPEND_SYSTEM.md` · utilizador: `<global>/APPEND_SYSTEM.md`.
pub const APPEND_SYSTEM_FILE: &str = "APPEND_SYSTEM.md";

/// Base do agente (identidade, disciplina e convenções do projecto).
///
/// É o **prompt do próprio agente**: as convenções do ktu vivem aqui (e não num
/// ficheiro gerado por projecto) — ver ADR 0022. O protocolo completo é o `ktu prime`.
pub const BASE: &str = "Es o ktu, um agente de programação. Trabalhas em passos: usa as tools para \
ler e alterar ficheiros e apresenta o teu raciocínio de forma concisa. Nunca inventes resultados: \
baseia-te no que as tools devolvem. Mantém memória durável do projeto: consulta `memory_ask` \
antes de decidir e grava cada aprendizado com `memory_write` — uma afirmação por nota, com \
`kind`, `tags` e `anchors` quando aplicáveis. É a memória que sobrevive entre sessões. \
Convenções do projecto: `.agents/` é o conhecimento (skills, guardrails, memória — versionável); \
`.ktu/` é o runtime (sessões, plugins, planos, trash); o `AGENTS.md` (raiz e aninhados) traz as \
instruções do projecto e entra no prompt. Protocolo completo: `ktu prime`.";

/// Compõe o system prompt: `base` + `AGENTS.md` condensado (se houver).
#[must_use]
pub fn compose(base: &str, agents_md: Option<&str>) -> String {
    compose_layers(base, agents_md, None, None)
}

/// Compõe o system prompt em **camadas**, por ordem: directriz (`system`),
/// `AGENTS.md`, skills e acrescento (`append`).
///
/// Cada camada é condensada e as vazias são omitidas; as restantes são separadas
/// por uma linha em branco. Determinístico e sem cache.
///
/// As skills ficam **antes** do acrescento: o `APPEND_SYSTEM.md` é a última palavra
/// do utilizador; as skills são conteúdo de referência (protocolos).
#[must_use]
pub fn compose_layers(
    system: &str,
    agents_md: Option<&str>,
    skills: Option<&str>,
    append: Option<&str>,
) -> String {
    let mut parts: Vec<String> = Vec::new();
    for layer in [Some(system), agents_md, skills, append]
        .into_iter()
        .flatten()
    {
        let text = condense(layer);
        if !text.is_empty() {
            parts.push(text);
        }
    }
    parts.join("\n\n")
}

/// Directriz de um nível (utilizador ou projecto): `SYSTEM.md` + `APPEND_SYSTEM.md`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Directive {
    system: Option<String>,
    append: Option<String>,
}

impl Directive {
    /// Carrega a directriz de `dir` (`SYSTEM.md` e `APPEND_SYSTEM.md`).
    ///
    /// Fail-open: ficheiro ausente, ilegível ou vazio ⇒ `None` (nunca erro).
    #[must_use]
    pub fn load(fs: &dyn Fs, dir: &Path) -> Self {
        Self {
            system: read(fs, &dir.join(SYSTEM_FILE)),
            append: read(fs, &dir.join(APPEND_SYSTEM_FILE)),
        }
    }

    /// Resolve dois níveis: o **projecto** tem precedência sobre o utilizador,
    /// ficheiro a ficheiro (homónimos não se combinam).
    #[must_use]
    pub fn resolve(user: Self, project: Self) -> Self {
        Self {
            system: project.system.or(user.system),
            append: project.append.or(user.append),
        }
    }

    /// Directriz que substitui a base, se declarada.
    #[must_use]
    pub fn system(&self) -> Option<&str> {
        self.system.as_deref()
    }

    /// Acrescento ao fim do prompt, se declarado.
    #[must_use]
    pub fn append(&self) -> Option<&str> {
        self.append.as_deref()
    }

    /// `true` se nenhum dos dois ficheiros contribui.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.system.is_none() && self.append.is_none()
    }
}

/// Lê um ficheiro de texto pela porta (fail-open: `None` se ausente/não-UTF-8/vazio).
fn read(fs: &dyn Fs, path: &Path) -> Option<String> {
    let bytes = fs.read(path).ok()?;
    let text = String::from_utf8(bytes).ok()?;
    (!text.trim().is_empty()).then_some(text)
}

/// Condensa markdown: remove blocos de comentário HTML e colapsa linhas em
/// branco. Puro e sem cache (o `AGENTS.md` é recomposto a cada turno).
#[must_use]
pub fn condense(text: &str) -> String {
    let mut lines: Vec<&str> = Vec::new();
    let mut in_comment = false;
    for raw in text.lines() {
        let line = raw.trim_end();
        let trimmed = line.trim_start();
        if in_comment {
            if trimmed.contains("-->") {
                in_comment = false;
            }
            continue;
        }
        if trimmed.starts_with("<!--") {
            if !trimmed.contains("-->") {
                in_comment = true;
            }
            continue;
        }
        lines.push(line);
    }
    let mut out: Vec<&str> = Vec::new();
    let mut blank = false;
    for line in lines {
        if line.trim().is_empty() {
            if !blank {
                out.push("");
            }
            blank = true;
        } else {
            out.push(line);
            blank = false;
        }
    }
    out.join("\n").trim().to_owned()
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{
        APPEND_SYSTEM_FILE, BASE, Directive, SYSTEM_FILE, compose, compose_layers, condense,
    };
    use crate::ports::MemFs;

    #[test]
    fn base_mentions_memory_discipline() {
        assert!(BASE.contains("memory_write"));
        assert!(BASE.contains("memory_ask"));
    }

    #[test]
    fn base_carries_the_project_conventions() {
        // As convenções do ktu vivem no prompt do agente, não num ficheiro gerado.
        for needle in [".agents/", ".ktu/", "AGENTS.md", "ktu prime"] {
            assert!(BASE.contains(needle), "falta {needle} na base");
        }
    }

    #[test]
    fn condense_drops_html_comments() {
        let text = "# titulo\n<!-- knudge:start -->\nsegredo\n<!-- knudge:end -->\ncorpo";
        let result = condense(text);
        assert!(!result.contains("knudge"));
        assert!(result.contains("corpo"));
    }

    #[test]
    fn condense_collapses_blank_runs() {
        assert_eq!(condense("a\n\n\n\nb"), "a\n\nb");
    }

    #[test]
    fn compose_without_agents_md_is_the_base() {
        assert_eq!(compose(BASE, None), BASE);
        assert_eq!(compose(BASE, Some("")), BASE);
    }

    #[test]
    fn compose_layers_orders_and_skips_empty_layers() {
        assert_eq!(
            compose_layers("sistema", Some("agents"), Some("skills"), Some("extra")),
            "sistema\n\nagents\n\nskills\n\nextra"
        );
        assert_eq!(compose_layers("sistema", None, None, None), "sistema");
        assert_eq!(
            compose_layers("sistema", Some("   "), None, Some("extra")),
            "sistema\n\nextra"
        );
    }

    #[test]
    fn directive_loads_both_files_and_is_fail_open() {
        let dir = Path::new("/cfg");
        assert!(Directive::load(&MemFs::new(), dir).is_empty());

        let fs = MemFs::new()
            .with_file(dir.join(SYSTEM_FILE), "nova base")
            .with_file(dir.join(APPEND_SYSTEM_FILE), "  ");
        let directive = Directive::load(&fs, dir);
        assert_eq!(directive.system(), Some("nova base"));
        assert_eq!(directive.append(), None);
        assert!(!directive.is_empty());
    }

    #[test]
    fn project_directive_wins_over_user() {
        let user = Directive {
            system: Some("user".to_owned()),
            append: None,
        };
        let project = Directive {
            system: None,
            append: Some("project".to_owned()),
        };
        let merged = Directive::resolve(user, project);
        assert_eq!(merged.system(), Some("user"));
        assert_eq!(merged.append(), Some("project"));

        let merged = Directive::resolve(
            Directive {
                system: Some("user".to_owned()),
                append: Some("user-extra".to_owned()),
            },
            Directive {
                system: Some("project".to_owned()),
                append: None,
            },
        );
        assert_eq!(merged.system(), Some("project"));
        assert_eq!(merged.append(), Some("user-extra"));
    }
}
