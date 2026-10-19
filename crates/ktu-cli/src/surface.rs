//! Superfície pública do binário (`ktu self surface`) — a medida do **teto** (H11).
//!
//! Conta o que o `ktu` expõe: **subcomandos**, **flags** e **tools**. É esta a
//! medida que `xtask check-surface` compara com `surface.toml`: a superfície não
//! cresce sem subir o teto numa alteração explícita (ver `wiki/adr/0011-*.md`).
//!
//! **Definição** (estável; mudá-la é mudar o teto):
//! - *subcomandos*: nós da árvore do `clap` (sem o verbo raiz, sem os escondidos);
//! - *flags*: `--longa`/`-c` declaradas em cada comando (os globais contam uma vez);
//! - *tools*: nomes distintos do catálogo builtin (`Toolbox` sem memória nem plano)
//!   mais as tools que o **loop** acrescenta (`subagent`, `tool_search`).

use std::collections::BTreeSet;
use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Arg, Command as ClapCommand, CommandFactory};
use ktu_core::plan::FsPlanHost;
use ktu_core::tools::ToolExecutor as _;
use ktu_core::{subagent, tool_search};
use ktu_memory::JsonlMemory;
use ktu_tools::{StdFs, Toolbox};

use crate::cli::Cli;
use crate::clock::SystemClock;
use crate::output::{self, Format};

/// Medida da superfície (ordem estável; é o `data` do envelope JSON).
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub(crate) struct Surface {
    /// Subcomandos visíveis (contando os aninhados).
    pub(crate) subcommands: usize,
    /// Flags distintas declaradas na árvore.
    pub(crate) flags: usize,
    /// Tools que o binário pode oferecer.
    pub(crate) tools: usize,
}

/// Mede a superfície: percorre a árvore do `clap` e conta o catálogo builtin.
pub(crate) fn measure() -> Surface {
    let root = Cli::command();
    let mut subcommands = 0;
    // As flags globais valem em toda a árvore: contam-se uma vez, no verbo raiz.
    let mut flags = root
        .get_arguments()
        .filter(|arg| is_flag(arg) && arg.is_global_set())
        .count();
    count(&root, &mut subcommands, &mut flags);
    Surface {
        subcommands,
        flags,
        tools: builtin_tools(),
    }
}

/// Imprime a superfície em `stdout` (texto ou envelope JSON).
pub(crate) fn run(format: Format) -> ExitCode {
    let surface = measure();
    let text = format!(
        "subcommands: {}\nflags: {}\ntools: {}\n",
        surface.subcommands, surface.flags, surface.tools
    );
    output::emit(format, "self surface", text.trim_end(), &surface);
    ExitCode::SUCCESS
}

/// Conta as flags **declaradas** em `command` (as globais já contaram no raiz) e desce.
fn count(command: &ClapCommand, subcommands: &mut usize, flags: &mut usize) {
    for arg in command.get_arguments().filter(|arg| is_flag(arg)) {
        if !arg.is_global_set() {
            *flags = flags.saturating_add(1);
        }
    }
    for sub in command.get_subcommands().filter(|sub| !sub.is_hide_set()) {
        *subcommands = subcommands.saturating_add(1);
        count(sub, subcommands, flags);
    }
}

/// Uma flag é um `--longa`/`-c` declarado (não posicional, não `--help`/`--version`).
fn is_flag(arg: &Arg) -> bool {
    if arg.is_positional() || matches!(arg.get_id().as_str(), "help" | "version") {
        return false;
    }
    arg.get_long().is_some() || arg.get_short().is_some()
}

/// Nomes distintos das tools que o binário pode oferecer (configuração máxima).
fn builtin_tools() -> usize {
    let root = PathBuf::from(".");
    let fs = StdFs;
    let clock = SystemClock::new();
    let memory = JsonlMemory::new(&fs, &clock, &root, root.join(".agents").join("memory"));
    let mutations = ktu_tools::FileMutationQueue::new();
    let toolbox = Toolbox::new(&root, &fs, &clock, Some(&memory), &mutations).with_plan(
        FsPlanHost::new(&fs, root.join(".ktu").join("plan"), "surface"),
    );
    let mut names: BTreeSet<String> = toolbox.specs().into_iter().map(|spec| spec.name).collect();
    // Tools que o **loop** acrescenta ao catálogo, fora do `Toolbox`: uma tool nova
    // neste nível tem de ser contada aqui (e o teto subido).
    names.insert(subagent::TOOL.to_owned());
    names.insert(tool_search::TOOL.to_owned());
    names.len()
}
