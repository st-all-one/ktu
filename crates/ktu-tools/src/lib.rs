//! `ktu-tools` — thread de execução de tools (T2): filesystem, `bash`, memória.
//!
//! Conhece os **efeitos**, nunca a sessão, o histórico nem o provider (§2.2 do
//! `plan.md`). Nenhuma tool corre sem `Guard::evaluate` em `Allow` (I10).
#![forbid(unsafe_code)]

pub mod bash;
pub mod fs;
pub mod fs_tools;
pub mod memory_tools;
mod path;
pub mod read_file;
mod search;
pub mod text;
pub mod tool_thread;
mod toolbox;
pub mod trash;
pub mod truncate;

pub mod agents;
pub use agents::{SUBAGENT_TOOL, default_registry, plan_mode_spec, read_only_tool_names};
pub use bash::BashTool;
pub use fs::StdFs;
pub use fs_tools::{FsTools, tool_names};
pub use memory_tools::MemoryTools;
pub use read_file::ReadFileTool;
pub use tool_thread::{ToolThread, WorkerCtx};
pub use toolbox::Toolbox;
pub use trash::TrashTool;

/// Nome canónico do crate (usado em logs estruturados).
pub const CRATE: &str = "ktu-tools";

#[cfg(test)]
mod tests {
    use super::CRATE;

    #[test]
    fn crate_name_is_reported() {
        assert_eq!(CRATE, "ktu-tools");
    }
}
