//! Fila de mutação por **caminho** (F4): serializa `write_file`/`edit_file` ao *mesmo*
//! ficheiro sem serializar caminhos distintos.
//!
//! A chave é o **caminho canónico** (o `realpath` segue *symlinks*; se o ficheiro ainda
//! não existir, o caminho já resolvido serve). Duas mutações ao mesmo ficheiro correm
//! **uma de cada vez** (ordem de chegada); duas a ficheiros diferentes correm em
//! paralelo. É uma fila de **borda** (T2): o kernel não conhece ficheiros (M2).
//!
//! Hoje a T2 serve um pedido de cada vez, pelo que não há escrita concorrente dentro
//! de uma sessão; a fila é a garantia que o paralelismo de tool calls vai precisar —
//! e é o que o gate G19 exercita.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, PoisonError};

use tokio::sync::{Mutex as AsyncMutex, OwnedMutexGuard};

/// A partir deste número de caminhos, a fila poda as entradas **sem ninguém à espera**.
const PRUNE_AT: usize = 256;

/// Fila de um caminho: o *lock* assíncrono que serializa as mutações.
type PathLock = Arc<AsyncMutex<()>>;

/// Fila de mutações, uma por caminho canónico.
#[derive(Default)]
pub struct FileMutationQueue {
    locks: Mutex<BTreeMap<PathBuf, PathLock>>,
}

impl FileMutationQueue {
    /// Cria uma fila vazia.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Espera a vez do caminho (ordem de chegada, uma mutação por caminho).
    ///
    /// A posse do *lock* é do chamador: enquanto o guarda viver, nenhuma outra
    /// mutação ao **mesmo** caminho avança.
    pub async fn acquire(&self, path: &Path) -> OwnedMutexGuard<()> {
        let key = canonical_key(path);
        let lock = {
            let mut locks = self.locks.lock().unwrap_or_else(PoisonError::into_inner);
            if locks.len() >= PRUNE_AT {
                locks.retain(|_, lock| Arc::strong_count(lock) > 1);
            }
            Arc::clone(locks.entry(key).or_default())
        };
        lock.lock_owned().await
    }

    /// Caminhos com fila activa (diagnóstico e testes).
    #[must_use]
    pub fn tracked(&self) -> usize {
        self.locks
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .len()
    }
}

/// Chave canónica do caminho: o `realpath` (segue *symlinks*), com *fallback* ao
/// próprio caminho quando o ficheiro ainda não existe (N2: sem `unwrap`).
fn canonical_key(path: &Path) -> PathBuf {
    std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}

#[cfg(test)]
mod tests {
    use std::path::Path;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::Duration;

    use super::FileMutationQueue;
    use tokio::sync::Barrier;
    use tokio::task::yield_now;
    use tokio::time::timeout;

    /// N tarefas ao **mesmo** caminho nunca estão dentro da secção crítica ao mesmo tempo.
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn the_same_path_serializes() {
        let queue = Arc::new(FileMutationQueue::new());
        let in_flight = Arc::new(AtomicUsize::new(0));
        let peak = Arc::new(AtomicUsize::new(0));
        let mut tasks = Vec::new();
        for _ in 0..8 {
            let queue = Arc::clone(&queue);
            let in_flight = Arc::clone(&in_flight);
            let peak = Arc::clone(&peak);
            tasks.push(tokio::spawn(async move {
                let _guard = queue.acquire(Path::new("/mesmo/ficheiro.txt")).await;
                let now = in_flight.fetch_add(1, Ordering::SeqCst).saturating_add(1);
                peak.fetch_max(now, Ordering::SeqCst);
                // Dá oportunidade ao escalonador de violar a fila (se estivesse partida).
                yield_now().await;
                in_flight.fetch_sub(1, Ordering::SeqCst);
            }));
        }
        for task in tasks {
            assert!(task.await.is_ok());
        }
        assert_eq!(
            peak.load(Ordering::SeqCst),
            1,
            "duas mutacoes ao mesmo tempo"
        );
        assert_eq!(queue.tracked(), 1);
    }

    /// Caminhos **distintos** não se esperam: as duas secções críticas sobrepõem-se.
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn distinct_paths_run_in_parallel() {
        let queue = Arc::new(FileMutationQueue::new());
        let barrier = Arc::new(Barrier::new(2));
        let mut tasks = Vec::new();
        for path in ["/um.txt", "/dois.txt"] {
            let queue = Arc::clone(&queue);
            let barrier = Arc::clone(&barrier);
            tasks.push(tokio::spawn(async move {
                let _guard = queue.acquire(Path::new(path)).await;
                // Se as duas filas fossem uma só, a primeira espera aqui para sempre.
                timeout(Duration::from_secs(5), barrier.wait())
                    .await
                    .is_ok()
            }));
        }
        for task in tasks {
            let overlapped = task.await.unwrap_or(false);
            assert!(overlapped, "caminhos distintos serializaram");
        }
        assert_eq!(queue.tracked(), 2);
    }
}
