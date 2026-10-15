//! Montagem do corpo do pedido para `/v1/chat/completions` (OpenAI-compatible).
//!
//! Funções **puras**: dado o pedido, devolvem o JSON. O `system` é composto
//! aqui (o log da sessão só tem o diálogo — ver `derive_messages`).

use ktu_core::domain::{Message, ToolCall, ToolSpec};
use ktu_core::provider::ProviderRequest;
use serde::Serialize;
use serde_json::Value;

use crate::sampling::{ReasoningEffort, Sampling};

#[derive(Serialize)]
struct WireFunction<'a> {
    name: &'a str,
    arguments: &'a str,
}

#[derive(Serialize)]
struct WireToolCall<'a> {
    id: &'a str,
    #[serde(rename = "type")]
    kind: &'static str,
    function: WireFunction<'a>,
}

#[derive(Serialize)]
struct WireMessage<'a> {
    role: &'static str,
    content: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_calls: Option<Vec<WireToolCall<'a>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_call_id: Option<&'a str>,
}

#[derive(Serialize)]
struct WireToolFunction<'a> {
    name: &'a str,
    description: &'a str,
    parameters: Value,
}

#[derive(Serialize)]
struct WireTool<'a> {
    #[serde(rename = "type")]
    kind: &'static str,
    function: WireToolFunction<'a>,
}

#[derive(Serialize)]
struct WireSampling {
    temperature: f32,
    top_p: f32,
    top_k: u32,
    min_p: f32,
    presence_penalty: f32,
    repetition_penalty: f32,
}

#[derive(Serialize)]
struct WireTemplateKwargs<'a> {
    enable_thinking: bool,
    reasoning_effort: &'a str,
}

#[derive(Serialize)]
struct WireRequest<'a> {
    model: &'a str,
    messages: Vec<WireMessage<'a>>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    tools: Vec<WireTool<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_choice: Option<&'static str>,
    stream: bool,
    #[serde(flatten)]
    sampling: WireSampling,
    chat_template_kwargs: WireTemplateKwargs<'a>,
}

/// Constrói o corpo JSON do pedido.
#[must_use]
pub fn request_body(
    model: &str,
    request: &ProviderRequest,
    sampling: Sampling,
    effort: ReasoningEffort,
) -> Value {
    let mut messages = Vec::new();
    if !request.system.is_empty() {
        messages.push(WireMessage {
            role: "system",
            content: &request.system,
            tool_calls: None,
            tool_call_id: None,
        });
    }
    messages.extend(request.messages.iter().map(message_wire));
    let tools: Vec<WireTool<'_>> = request.tools.iter().map(tool_wire).collect();
    let tool_choice = if tools.is_empty() { None } else { Some("auto") };
    let body = WireRequest {
        model,
        messages,
        tools,
        tool_choice,
        stream: true,
        sampling: WireSampling {
            temperature: sampling.temperature,
            top_p: sampling.top_p,
            top_k: sampling.top_k,
            min_p: sampling.min_p,
            presence_penalty: sampling.presence_penalty,
            repetition_penalty: sampling.repetition_penalty,
        },
        chat_template_kwargs: WireTemplateKwargs {
            enable_thinking: true,
            reasoning_effort: effort.as_str(),
        },
    };
    to_value(&body)
}

/// Serializa um valor para JSON (não falha para os tipos do wire).
fn to_value<T: Serialize>(value: &T) -> Value {
    serde_json::to_value(value).unwrap_or(Value::Null)
}

fn tool_call_wire(call: &ToolCall) -> WireToolCall<'_> {
    WireToolCall {
        id: call.id.as_str(),
        kind: "function",
        function: WireFunction {
            name: &call.name,
            arguments: &call.arguments,
        },
    }
}

fn message_wire(message: &Message) -> WireMessage<'_> {
    match message {
        Message::System { content } => WireMessage {
            role: "system",
            content,
            tool_calls: None,
            tool_call_id: None,
        },
        Message::User { content } => WireMessage {
            role: "user",
            content,
            tool_calls: None,
            tool_call_id: None,
        },
        Message::Assistant {
            content,
            tool_calls,
            ..
        } => WireMessage {
            role: "assistant",
            content,
            tool_calls: if tool_calls.is_empty() {
                None
            } else {
                Some(tool_calls.iter().map(tool_call_wire).collect())
            },
            tool_call_id: None,
        },
        Message::Tool {
            tool_call_id,
            content,
            ..
        } => WireMessage {
            role: "tool",
            content,
            tool_calls: None,
            tool_call_id: Some(tool_call_id.as_str()),
        },
    }
}

fn tool_wire(spec: &ToolSpec) -> WireTool<'_> {
    let parameters = serde_json::from_str(&spec.parameters)
        .unwrap_or_else(|_| serde_json::json!({"type": "object", "properties": {}}));
    WireTool {
        kind: "function",
        function: WireToolFunction {
            name: &spec.name,
            description: &spec.description,
            parameters,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::request_body;
    use crate::sampling::{ReasoningEffort, Sampling};
    use ktu_core::cancel::CancelToken;
    use ktu_core::domain::{Message, ToolCall, ToolCallId, ToolSpec};
    use ktu_core::provider::ProviderRequest;

    fn request(messages: Vec<Message>, tools: Vec<ToolSpec>) -> ProviderRequest {
        ProviderRequest {
            system: "se".to_owned(),
            messages,
            tools,
            cancel: CancelToken::new(),
        }
    }

    #[test]
    fn body_has_system_sampling_and_stream() {
        let body = request_body(
            "ornith",
            &request(
                vec![Message::User {
                    content: "ola".to_owned(),
                }],
                Vec::new(),
            ),
            Sampling::ornith(),
            ReasoningEffort::Xhigh,
        );
        assert_eq!(body.get("stream"), Some(&serde_json::json!(true)));
        assert_eq!(body.get("top_k"), Some(&serde_json::json!(20)));
        assert_eq!(
            body.pointer("/chat_template_kwargs/reasoning_effort"),
            Some(&serde_json::json!("xhigh"))
        );
        let messages = body.get("messages").and_then(|value| value.as_array());
        assert_eq!(messages.map(Vec::len), Some(2));
    }

    #[test]
    fn assistant_tool_calls_use_one_message_with_n_calls() {
        let body = request_body(
            "m",
            &request(
                vec![Message::Assistant {
                    content: String::new(),
                    tool_calls: vec![
                        ToolCall {
                            id: ToolCallId::new("a"),
                            name: "read_file".to_owned(),
                            arguments: "{}".to_owned(),
                        },
                        ToolCall {
                            id: ToolCallId::new("b"),
                            name: "read_file".to_owned(),
                            arguments: "{}".to_owned(),
                        },
                    ],
                }],
                vec![ToolSpec {
                    name: "read_file".to_owned(),
                    description: "le".to_owned(),
                    parameters: "{\"type\":\"object\"}".to_owned(),
                }],
            ),
            Sampling::ornith(),
            ReasoningEffort::Medium,
        );
        let messages = body
            .get("messages")
            .and_then(|value| value.as_array())
            .map(Vec::len);
        assert_eq!(messages, Some(2));
        let calls = body
            .pointer("/messages/1/tool_calls")
            .and_then(|value| value.as_array())
            .map(Vec::len);
        assert_eq!(calls, Some(2));
        assert_eq!(
            body.pointer("/tools/0/function/name"),
            Some(&serde_json::json!("read_file"))
        );
        assert_eq!(body.get("tool_choice"), Some(&serde_json::json!("auto")));
    }
}
