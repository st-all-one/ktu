use super::{Flavor, request_body};
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
        Flavor::Llama,
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
        Flavor::Llama,
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

#[test]
fn openai_flavor_omits_llama_only_fields() {
    let body = request_body(
        "m",
        &request(
            vec![Message::User {
                content: "ola".to_owned(),
            }],
            Vec::new(),
        ),
        Sampling::ornith(),
        ReasoningEffort::Xhigh,
        Flavor::OpenAi,
    );
    assert_eq!(body.get("temperature"), Some(&serde_json::json!(1.0)));
    assert!(body.get("top_k").is_none());
    assert!(body.get("min_p").is_none());
    assert!(body.get("presence_penalty").is_none());
    assert!(body.get("repetition_penalty").is_none());
    assert!(
        body.pointer("/chat_template_kwargs/reasoning_effort")
            .is_none()
    );
}
