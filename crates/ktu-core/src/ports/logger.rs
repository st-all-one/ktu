//! Porta de log: a única forma de o domínio emitir eventos estruturados.
//!
//! **stdout = dados, stderr = logs** (§1.3 do `epic.md`). O core **não** conhece
//! `tracing`: a implementação real fica na borda (`ktu-cli`); os testes usam os
//! *fakes*. Os nomes de evento são **[`crate::events`] ids estáveis**, nunca
//! texto livre (contrato de máquina).

/// Nível de log.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Level {
    /// Falha que impede a operação.
    Error,
    /// Situação anómala; a operação segue.
    Warn,
    /// Marco de progresso normal.
    Info,
    /// Detalhe de diagnóstico.
    Debug,
    /// Rastreamento fino.
    Trace,
}

/// Registro estruturado de log.
#[derive(Debug)]
pub struct LogRecord<'a> {
    /// Nível.
    pub level: Level,
    /// Id estável do evento (ver [`crate::events`]).
    pub event: &'a str,
    /// Mensagem legível (nunca contém segredos nem corpo de ficheiro).
    pub message: &'a str,
    /// Campos estruturados `(chave, valor)`.
    pub fields: &'a [(&'a str, &'a str)],
}

/// Destino de log. Nunca deve panicar nem escrever em `stdout`.
pub trait Logger: Send + Sync {
    /// Emite um registro.
    fn log(&self, record: &LogRecord<'_>);
}
