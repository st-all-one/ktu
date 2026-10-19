//! Envelope de saída (`--json`) — o **contrato de máquina** do CLI.
//!
//! Sem `--json`, `stdout` recebe o dado em texto; com `--json`, recebe
//! `{success, command, data?, error?}`. Os **logs** continuam em `stderr`
//! (nunca se misturam). O envelope é estável e aditivo.

use std::io::{self, Write};

use serde::Serialize;

/// Modo de saída: texto humano ou envelope JSON.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Format {
    /// Texto em `stdout` (humano).
    Text,
    /// Envelope JSON em `stdout` (máquina).
    Json,
}

/// Envelope de sucesso.
#[derive(Debug, Serialize)]
#[allow(
    clippy::struct_field_names,
    reason = "`success` é a chave fixa do contrato JSON, não um nome livre"
)]
pub(crate) struct Success<'a, T> {
    /// Sempre `true` no sucesso.
    pub success: bool,
    /// Verbo executado (ex.: `prime`).
    pub command: &'a str,
    /// Dados específicos do verbo.
    pub data: T,
}

/// Envelope de erro.
#[derive(Debug, Serialize)]
#[allow(
    clippy::struct_field_names,
    reason = "`success` é a chave fixa do contrato JSON, não um nome livre"
)]
pub(crate) struct Failure<'a> {
    /// Sempre `false` no erro.
    pub success: bool,
    /// Verbo executado.
    pub command: &'a str,
    /// Erro tipado.
    pub error: ErrorInfo<'a>,
}

/// Informação de erro tipada (contrato de máquina).
#[derive(Debug, Serialize)]
pub(crate) struct ErrorInfo<'a> {
    /// Código estável (`invalid`, `not_found`, `io`, …).
    pub code: &'a str,
    /// Mensagem legível.
    pub message: &'a str,
}

/// Imprime dados: em texto (`text`) ou no envelope JSON (`data`).
pub(crate) fn emit<T: Serialize>(format: Format, command: &str, text: &str, data: &T) {
    match format {
        Format::Json => print_json(&Success {
            success: true,
            command,
            data,
        }),
        Format::Text => {
            let mut stdout = io::stdout().lock();
            let _ignored = writeln!(stdout, "{text}");
        }
    }
}

/// Reporta um erro: no envelope JSON (se `Json`) ou em `stderr` (texto).
pub(crate) fn emit_error(format: Format, command: &str, code: &str, message: &str) {
    match format {
        Format::Json => print_json(&Failure {
            success: false,
            command,
            error: ErrorInfo { code, message },
        }),
        Format::Text => {
            let mut stderr = io::stderr().lock();
            let _ignored = writeln!(stderr, "ktu: {message}");
        }
    }
}

/// Escreve um **aviso** em `stderr` (log): `stdout` é só dados.
pub(crate) fn warn(message: &str) {
    let mut stderr = io::stderr().lock();
    let _ignored = writeln!(stderr, "ktu: {message}");
}

/// Serializa um valor para `stdout` como uma linha JSON.
fn print_json<T: Serialize>(value: &T) {
    let stdout = io::stdout();
    let mut lock = stdout.lock();
    if serde_json::to_writer(&mut lock, value).is_ok() {
        let _ignored = writeln!(lock);
    }
}

#[cfg(test)]
mod tests {
    use super::{Failure, Success};

    #[test]
    fn success_envelope_has_the_machine_shape() {
        let envelope = Success {
            success: true,
            command: "prime",
            data: serde_json::json!({ "protocol": "x" }),
        };
        let text = serde_json::to_string(&envelope).unwrap_or_default();
        assert!(text.contains("\"success\":true"));
        assert!(text.contains("\"command\":\"prime\""));
        assert!(text.contains("\"data\""));
    }

    #[test]
    fn failure_envelope_is_typed() {
        let envelope = Failure {
            success: false,
            command: "run",
            error: super::ErrorInfo {
                code: "invalid",
                message: "prompt vazio",
            },
        };
        let text = serde_json::to_string(&envelope).unwrap_or_default();
        assert!(text.contains("\"success\":false"));
        assert!(text.contains("\"code\":\"invalid\""));
    }
}
