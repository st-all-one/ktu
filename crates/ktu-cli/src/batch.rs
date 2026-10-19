//! `--batch <JSONL>` — lote de turnos independentes.
//!
//! Cada linha é um objeto `--params`. **Paralelo** por omissão (turnos efémeros,
//! cada um com sessão própria, concorrência limitada); `--sequential` corre tudo
//! na **mesma sessão**, em ordem. Os resultados são impressos **ordenados pelo
//! índice** (determinismo).

use std::io::{self, Read, Write};
use std::path::Path;
use std::process::ExitCode;
use std::sync::atomic::{AtomicUsize, Ordering};

use ktu_tools::FileMutationQueue;
use std::sync::Arc;
use std::sync::{Mutex, PoisonError};

use ktu_core::ports::Logger;

use crate::params::{self, RunParams};
use crate::run::{self, RunConfig};
use crate::session_mode::{SessionMode, SessionTarget};

/// Resultado de um turno do lote: `(sucesso, resposta final)`.
type TurnResult = (bool, Option<String>);

/// Resultados indexados (preserva a ordem de entrada).
type Results = Mutex<Vec<Option<TurnResult>>>;

/// Opções do lote.
pub(crate) struct BatchOptions<'a> {
    /// Fonte do JSONL (`-` lê de `stdin`).
    pub source: &'a str,
    /// Corre tudo na mesma sessão, em ordem.
    pub sequential: bool,
    /// Concorrência máxima (modo paralelo).
    pub concurrency: usize,
}

/// Lê o lote e corre os turnos.
pub(crate) fn run(
    root: &Path,
    logger: &dyn Logger,
    base: &RunConfig,
    options: &BatchOptions<'_>,
) -> ExitCode {
    let items = match read_items(options.source) {
        Ok(items) => items,
        Err(message) => {
            run::report_error(&message);
            return ExitCode::FAILURE;
        }
    };
    if items.is_empty() {
        return ExitCode::SUCCESS;
    }
    // **Uma** fila por processo: os workers do modo paralelo partilham-na, pelo que
    // duas entradas a editar o mesmo ficheiro não perdem alterações (F4).
    let mutations = Arc::new(FileMutationQueue::new());
    let ok = if options.sequential {
        run_sequential(root, logger, base, &items, &mutations)
    } else {
        run_parallel(root, logger, base, &items, options.concurrency, &mutations)
    };
    if ok {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

/// Lê um JSONL de objetos `--params` (linhas vazias ignoradas).
fn read_items(source: &str) -> Result<Vec<RunParams>, String> {
    let text = if source == "-" {
        let mut buffer = String::new();
        io::stdin()
            .read_to_string(&mut buffer)
            .map_err(|error| format!("nao foi possivel ler stdin: {error}"))?;
        buffer
    } else {
        std::fs::read_to_string(source)
            .map_err(|error| format!("nao foi possivel ler o lote: {error}"))?
    };
    let mut items = Vec::new();
    for (index, line) in text.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let line_number = index.saturating_add(1);
        let item =
            params::parse(trimmed).map_err(|error| format!("linha {line_number}: {error}"))?;
        items.push(item);
    }
    Ok(items)
}

/// Constrói a config de um item, sobrepondo o `--params` à base.
///
/// Em modo **sequencial** o item não é o primeiro, a sessão é **retomada**: os
/// turnos partilham o contexto (é o que "uma sessão, N turnos" quer dizer).
fn turn_config(base: &RunConfig, item: &RunParams, session: &str, mode: SessionMode) -> RunConfig {
    let mut config = base.clone();
    if let Some(provider) = &item.provider {
        config.provider.clone_from(provider);
    }
    if let Some(base_url) = &item.base_url {
        config.base_url.clone_from(base_url);
    }
    if let Some(model) = &item.model {
        config.model.clone_from(model);
    }
    if let Some(api_key) = &item.api_key {
        config.api_key = Some(api_key.clone());
    }
    if let Some(effort) = &item.effort {
        config.effort.clone_from(effort);
    }
    if let Some(max_steps) = item.max_steps {
        config.max_steps = max_steps;
    }
    let id = item.session.clone().unwrap_or_else(|| session.to_owned());
    config.session = SessionTarget::Persistent(id);
    config.mode = mode;
    config
}

/// Corre os itens em ordem, na **mesma sessão**.
fn run_sequential(
    root: &Path,
    logger: &dyn Logger,
    base: &RunConfig,
    items: &[RunParams],
    mutations: &Arc<FileMutationQueue>,
) -> bool {
    let mut ok = true;
    for (index, item) in items.iter().enumerate() {
        // O 1.º item usa o modo pedido; os seguintes **retomam** (contexto acumula).
        let mode = if index == 0 {
            base.mode
        } else {
            SessionMode::Resume
        };
        let config = turn_config(base, item, base.session.id().unwrap_or("batch"), mode);
        let prompt = item.prompt.clone().unwrap_or_default();
        let (success, text) = run::run_capture(root, &prompt, logger, &config, mutations);
        print_result(index, text.as_deref());
        ok = ok && success;
    }
    ok
}

/// Corre os itens em paralelo, cada um numa sessão própria (`batch-<i>`).
#[allow(
    clippy::too_many_arguments,
    reason = "composicao explicita do lote; uma fila partilhada + o contexto do turno"
)]
fn run_parallel(
    root: &Path,
    logger: &dyn Logger,
    base: &RunConfig,
    items: &[RunParams],
    concurrency: usize,
    mutations: &Arc<FileMutationQueue>,
) -> bool {
    let total = items.len();
    let results: Results = Mutex::new((0..total).map(|_| None).collect());
    let counter = AtomicUsize::new(0);
    let workers = concurrency.max(1).min(total);
    std::thread::scope(|scope| {
        for _ in 0..workers {
            scope.spawn(|| {
                loop {
                    let index = counter.fetch_add(1, Ordering::SeqCst);
                    let Some(item) = items.get(index) else {
                        break;
                    };
                    let config = turn_config(base, item, &format!("batch-{index}"), base.mode);
                    let prompt = item.prompt.clone().unwrap_or_default();
                    let (success, text) =
                        run::run_capture(root, &prompt, logger, &config, mutations);
                    let mut guard = results.lock().unwrap_or_else(PoisonError::into_inner);
                    if let Some(slot) = guard.get_mut(index) {
                        *slot = Some((success, text));
                    }
                }
            });
        }
    });
    let mut ok = true;
    let guard = results.lock().unwrap_or_else(PoisonError::into_inner);
    for (index, entry) in guard.iter().enumerate() {
        if let Some((success, text)) = entry {
            print_result(index, text.as_deref());
            ok = ok && *success;
        }
    }
    ok
}

/// Imprime o resultado de um item (resposta final ou marcador de vazio).
fn print_result(index: usize, text: Option<&str>) {
    let mut stdout = io::stdout().lock();
    let content = text.unwrap_or("(sem resposta)");
    let _ignored = writeln!(stdout, "[{index}] {content}");
}

#[cfg(test)]
mod tests {
    use super::{RunParams, turn_config};
    use crate::run::RunConfig;
    use crate::session_mode::{SessionMode, SessionTarget};

    fn base() -> RunConfig {
        RunConfig {
            script: None,
            provider: "llama".to_owned(),
            base_url: "http://localhost".to_owned(),
            model: "m".to_owned(),
            api_key: None,
            effort: "xhigh".to_owned(),
            max_steps: 24,
            deferred_tools: Vec::new(),
            session: SessionTarget::Persistent("default".to_owned()),
            mode: SessionMode::Fresh,
            plan: false,
        }
    }

    #[test]
    fn item_overrides_the_base() {
        let item = RunParams {
            provider: Some("mock".to_owned()),
            max_steps: Some(3),
            ..RunParams::default()
        };
        let config = turn_config(&base(), &item, "s", SessionMode::Fresh);
        assert_eq!(config.provider, "mock");
        assert_eq!(config.max_steps, 3);
        assert_eq!(config.session, SessionTarget::Persistent("s".to_owned()));
    }

    #[test]
    fn item_without_overrides_keeps_the_base() {
        let config = turn_config(&base(), &RunParams::default(), "s", SessionMode::Fresh);
        assert_eq!(config.provider, "llama");
        assert_eq!(config.session, SessionTarget::Persistent("s".to_owned()));
    }
}
