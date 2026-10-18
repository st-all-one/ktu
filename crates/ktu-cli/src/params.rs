//! `--params <JSON>` — o **contrato de máquina** do turno.
//!
//! Um objeto JSON define todos os parâmetros do turno; as **flags explícitas têm
//! precedência** (`flag > --params > default`). `-` lê o objeto de `stdin`.

use std::io::{self, Read};

use serde::Deserialize;

/// Parâmetros do turno vindos de `--params` (todos opcionais).
#[derive(Debug, Default, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub(crate) struct RunParams {
    /// Instrução (usada quando o argumento posicional está vazio).
    pub prompt: Option<String>,
    /// Provider (`llama`|`opencode-go`|`mock`).
    pub provider: Option<String>,
    /// *Base URL* do provider.
    pub base_url: Option<String>,
    /// Nome/alias do modelo.
    pub model: Option<String>,
    /// Chave do gateway remoto.
    pub api_key: Option<String>,
    /// Esforço de raciocínio (`low|medium|high|xhigh`).
    pub effort: Option<String>,
    /// Tecto de passos do turno.
    pub max_steps: Option<u32>,
    /// Identificador da sessão.
    pub session: Option<String>,
    /// Retoma a sessão antes do turno.
    pub resume: Option<bool>,
    /// Força a compactação antes do turno.
    pub compact: Option<bool>,
    /// Arranca em modo plano (read-only).
    pub plan: Option<bool>,
}

/// Lê `--params` (JSON); `-` lê o objeto de `stdin`.
///
/// # Errors
/// Devolve mensagem legível se o JSON for inválido ou a leitura falhar.
pub(crate) fn parse(source: &str) -> Result<RunParams, String> {
    let text = if source == "-" {
        let mut buffer = String::new();
        io::stdin()
            .read_to_string(&mut buffer)
            .map_err(|error| format!("nao foi possivel ler stdin: {error}"))?;
        buffer
    } else {
        source.to_owned()
    };
    serde_json::from_str(&text).map_err(|error| format!("--params invalido: {error}"))
}

/// Funde flag explícita > `--params` > default.
pub(crate) fn pick<T>(flag: Option<T>, param: Option<T>, default: T) -> T {
    flag.or(param).unwrap_or(default)
}

#[cfg(test)]
mod tests {
    use super::{RunParams, parse, pick};

    #[test]
    fn parses_a_full_object() {
        let params =
            parse(r#"{"prompt":"faz x","provider":"mock","max_steps":3}"#).unwrap_or_default();
        assert_eq!(params.prompt.as_deref(), Some("faz x"));
        assert_eq!(params.provider.as_deref(), Some("mock"));
        assert_eq!(params.max_steps, Some(3));
    }

    #[test]
    fn empty_object_is_all_defaults() {
        let params = parse("{}").unwrap_or_default();
        assert_eq!(params, RunParams::default());
    }

    #[test]
    fn rejects_unknown_fields() {
        assert!(parse(r#"{"nope":1}"#).is_err());
    }

    #[test]
    fn pick_prefers_flag_then_param_then_default() {
        assert_eq!(
            pick(Some("a".to_owned()), Some("b".to_owned()), "d".to_owned()),
            "a"
        );
        assert_eq!(pick(None, Some("b".to_owned()), "d".to_owned()), "b");
        assert_eq!(pick(None, None, "d".to_owned()), "d");
    }
}
