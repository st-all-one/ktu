//! Portas do núcleo — a fronteira do efeito.
//!
//! O domínio é **puro**: nunca lê o relógio global, o ambiente, o sistema de
//! ficheiros, o gerador aleatório nem o `tracing`. Esses efeitos entram por
//! **portas** (traits) injectadas; a borda fornece adaptadores reais e os
//! testes fornecem *fakes* determinísticos ([`FixedClock`], [`MemFs`], [`FakeEmbedder`], …).

mod embedder;
mod fakes;
mod instructions;
mod logger;
mod metrics;

#[cfg(test)]
mod tests;

use std::path::Path;

use crate::Result;

pub use embedder::{EmbedError, Embedder, EmbeddingMeta};
pub use fakes::{
    FakeEmbedder, FixedClock, MapEnv, MapInstructions, MemFs, NoopLogger, NoopMetrics, OwnedRecord,
    RecordingLogger, RecordingMetrics, SeqClock, SeqRng, UnavailableEmbedder,
};
pub use instructions::{Instructions, Local};
pub use logger::{Level, LogRecord, Logger};
pub use metrics::{Metric, Metrics, MetricsSnapshot};

/// Instante em milissegundos desde a época Unix (UTC), como **dado**.
///
/// O domínio nunca usa `SystemTime` (proibido pelo `clippy.toml`); o tempo chega
/// pela porta [`Clock`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Timestamp(i64);

impl Timestamp {
    /// Constrói um instante a partir de milissegundos desde a época.
    #[must_use]
    pub const fn from_millis(millis: i64) -> Self {
        Self(millis)
    }

    /// Devolve os milissegundos desde a época.
    #[must_use]
    pub const fn as_millis(self) -> i64 {
        self.0
    }
}

/// Porta de tempo: a única fonte do instante corrente e de leituras monótonas.
pub trait Clock: Send + Sync {
    /// Instante corrente (relógio de parede).
    fn now(&self) -> Timestamp;

    /// Leitura monótona em microssegundos, para medição de duração por função.
    fn monotonic_micros(&self) -> u64;
}

/// Porta de aleatoriedade: a única fonte de valores pseudo-aleatórios.
pub trait Rng: Send + Sync {
    /// Próximo valor de 64 bits.
    fn next_u64(&self) -> u64;
}

/// Porta de sistema de ficheiros.
///
/// O adaptador real confina cada operação ao *root* da sessão; a porta em si
/// trata o caminho como dado.
pub trait Fs: Send + Sync {
    /// Lê o conteúdo completo de um ficheiro.
    fn read(&self, path: &Path) -> Result<Vec<u8>>;

    /// Escreve o conteúdo, criando ou truncando o ficheiro.
    fn write(&self, path: &Path, contents: &[u8]) -> Result<()>;

    /// Indica se o caminho existe.
    fn exists(&self, path: &Path) -> bool;

    /// Lista as entradas directas de um directório.
    fn read_dir(&self, path: &Path) -> Result<Vec<std::path::PathBuf>>;

    /// Resolve o caminho **real** (segue *symlinks*); erro se não existir.
    fn canonicalize(&self, path: &Path) -> Result<std::path::PathBuf>;

    /// Indica se o caminho é um directório.
    fn is_dir(&self, path: &Path) -> bool;

    /// Renomeia (movimento atómico no mesmo volume) — para escritas duráveis.
    fn rename(&self, from: &Path, to: &Path) -> Result<()>;

    /// Força os bytes de um ficheiro (ou a entrada de um directório) para o disco
    /// (`fsync`). Num sistema em memória é um no-op.
    fn sync(&self, path: &Path) -> Result<()>;
}

/// Porta de ambiente: variáveis de ambiente como dado injectado.
pub trait Env: Send + Sync {
    /// Valor da variável, se definida.
    fn var(&self, key: &str) -> Option<String>;
}
