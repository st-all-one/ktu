//! `ktu prompt` — mostra o **system prompt efectivo** do projecto.
//!
//! É o mesmo texto que a fachada entrega ao modelo: base ou `.agents/SYSTEM.md`,
//! `AGENTS.md` (raiz + aninhados) e `.agents/APPEND_SYSTEM.md`, com a directriz do
//! utilizador (`~/.config/ktu/`) como nível inferior. Serve para diagnosticar
//! convenções ("porque é que o modelo não vê a minha regra?").

use std::path::PathBuf;
use std::process::ExitCode;

use ktu_core::facade::system_prompt_in;
use ktu_tools::StdFs;

use crate::config_cmd::global_dir_path;
use crate::output::{self, Format};
use crate::root_or_cwd;

/// Imprime o system prompt efectivo em `stdout` (dado, não log).
pub(crate) fn show(root: Option<PathBuf>, format: Format) -> ExitCode {
    let root = root_or_cwd(root);
    let fs = StdFs;
    let user_dir = global_dir_path();
    let system = system_prompt_in(&fs, &root, user_dir.as_deref());
    let data = serde_json::json!({
        "system": system,
        "bytes": system.len(),
        "user_dir": user_dir.map(|dir| dir.display().to_string()),
    });
    output::emit(format, "prompt", &system, &data);
    ExitCode::SUCCESS
}
