//! Integração com o git gerida por `ktu init` — **pura** (texto → texto, sem I/O).
//!
//! O ktu **nunca** escreve `.gitignore` (o conhecimento pertence ao projecto): a
//! exclusão vive em `.git/info/exclude` e as regras em `.gitattributes`. Os blocos
//! são delimitados por marcadores e cada operação é **idempotente**.

pub mod attributes;
pub mod block;
pub mod exclude;

/// Política de versionamento do conhecimento no git.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GitMode {
    /// `--git-tracked`: versiona `.agents/`; exclui só o derivado e o runtime.
    Tracked,
    /// `--git-excluded`: exclui `.agents/` e `.ktu/` (local-only).
    Excluded,
}
