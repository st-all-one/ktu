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
    #[serde(skip_serializing_if = "Option::is_none")]
    top_k: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    min_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    presence_penalty: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    repetition_penalty: Option<f32>,
}

/// Dialeto do corpo: o `llama-server` aceita extensões; um gateway `OpenAI` puro, não.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Flavor {
    /// Inclui as extensões do llama.cpp (`top_k`, `min_p`, `presence/repetition_penalty`, `reasoning_effort`).
    Llama,
    /// Só os campos OpenAI-padrão (gateways remotos toleram mal os extras).
    OpenAi,
}

#[derive(Serialize)]
struct WireTemplateKwargs<'a> {
    enable_thinking: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    reasoning_effort: Option<&'a str>,
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
    flavor: Flavor,
) -> Value {
    let llama = flavor == Flavor::Llama;
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
            top_k: llama.then_some(sampling.top_k),
            min_p: llama.then_some(sampling.min_p),
            presence_penalty: llama.then_some(sampling.presence_penalty),
            repetition_penalty: llama.then_some(sampling.repetition_penalty),
        },
        chat_template_kwargs: WireTemplateKwargs {
            enable_thinking: true,
            reasoning_effort: llama.then_some(effort.as_str()),
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
mod tests;
