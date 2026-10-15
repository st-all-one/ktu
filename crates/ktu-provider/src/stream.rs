//! Acumulador de deltas do *streaming* para uma [`Completion`].
//!
//! O `delta` do `llama-server` traz `content`, `reasoning_content` (pensamento)
//! e `tool_calls` (fragmentos por `index`, concatenados). O pensamento é
//! acumulado num campo próprio — nunca misturado com a resposta.

use std::collections::BTreeMap;

use ktu_core::domain::{Completion, ToolCall, ToolCallId};
use ktu_core::provider::ProviderError;
use serde_json::Value;

/// Fragmento de uma chamada de tool em construção.
#[derive(Debug, Default)]
struct CallBuilder {
    id: String,
    name: String,
    arguments: String,
}

/// Acumula os `delta` de um passo do modelo.
#[derive(Debug, Default)]
pub struct CompletionAccumulator {
    text: String,
    reasoning: String,
    calls: BTreeMap<u64, CallBuilder>,
}

impl CompletionAccumulator {
    /// Cria um acumulador vazio.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Aplica um *payload* `data:` (um objeto JSON de `choices[].delta`).
    ///
    /// `[DONE]` é ignorado (o fim da stream é o fim do corpo HTTP).
    pub fn apply(&mut self, payload: &str) -> Result<(), ProviderError> {
        if payload == "[DONE]" {
            return Ok(());
        }
        let value: Value = serde_json::from_str(payload)
            .map_err(|error| ProviderError::Invalid(format!("sse invalido: {error}")))?;
        let Some(delta) = value
            .get("choices")
            .and_then(Value::as_array)
            .and_then(|choices| choices.first())
            .and_then(|choice| choice.get("delta"))
        else {
            return Ok(());
        };
        append_field(&mut self.text, delta.get("content"));
        append_field(&mut self.reasoning, delta.get("reasoning_content"));
        append_field(&mut self.reasoning, delta.get("reasoning"));
        self.apply_tool_calls(delta.get("tool_calls"));
        Ok(())
    }

    fn apply_tool_calls(&mut self, calls: Option<&Value>) {
        let Some(entries) = calls.and_then(Value::as_array) else {
            return;
        };
        for entry in entries {
            let index = entry.get("index").and_then(Value::as_u64).unwrap_or(0);
            let builder = self.calls.entry(index).or_default();
            if let Some(id) = entry.get("id").and_then(Value::as_str) {
                id.clone_into(&mut builder.id);
            }
            if let Some(function) = entry.get("function") {
                if let Some(name) = function.get("name").and_then(Value::as_str) {
                    builder.name.push_str(name);
                }
                append_field(&mut builder.arguments, function.get("arguments"));
            }
        }
    }

    /// Fecha e devolve a [`Completion`] acumulada.
    #[must_use]
    pub fn finish(self) -> Completion {
        let tool_calls = self
            .calls
            .into_iter()
            .map(|(index, builder)| ToolCall {
                id: if builder.id.is_empty() {
                    ToolCallId::new(format!("call_{index}"))
                } else {
                    ToolCallId::new(builder.id)
                },
                name: builder.name,
                arguments: builder.arguments,
            })
            .collect();
        Completion {
            text: self.text,
            reasoning: self.reasoning,
            tool_calls,
        }
    }
}

/// Acrescenta o texto de um campo JSON, se for string.
fn append_field(target: &mut String, value: Option<&Value>) {
    if let Some(text) = value.and_then(Value::as_str) {
        target.push_str(text);
    }
}

#[cfg(test)]
mod tests {
    use super::CompletionAccumulator;

    #[test]
    fn accumulates_content_reasoning_and_tool_fragments() {
        let mut acc = CompletionAccumulator::new();
        let payloads = [
            r#"{"choices":[{"delta":{"reasoning_content":"penso "}}]}"#,
            r#"{"choices":[{"delta":{"content":"ola"}}]}"#,
            r#"{"choices":[{"delta":{"tool_calls":[{"index":0,"id":"c1","function":{"name":"read_file","arguments":"{\"pa"}}]}}]}"#,
            r#"{"choices":[{"delta":{"tool_calls":[{"index":0,"function":{"arguments":"th\":\"a\"}"}}]}}]}"#,
            "[DONE]",
        ];
        for payload in payloads {
            let result = acc.apply(payload);
            assert!(result.is_ok());
        }
        let completion = acc.finish();
        assert_eq!(completion.text, "ola");
        assert_eq!(completion.reasoning, "penso ");
        assert_eq!(completion.tool_calls.len(), 1);
        assert_eq!(
            completion.tool_calls.first().map(|c| c.name.as_str()),
            Some("read_file")
        );
        assert_eq!(
            completion.tool_calls.first().map(|c| c.arguments.as_str()),
            Some(r#"{"path":"a"}"#)
        );
    }

    #[test]
    fn rejects_invalid_json() {
        let mut acc = CompletionAccumulator::new();
        assert!(acc.apply("nao e json").is_err());
    }
}
