//! Integração da tool `bash`: execução, análise de argv e timeout.

use std::time::Duration;

use ktu_core::domain::{ToolCall, ToolCallId};
use ktu_core::tools::{ToolExecutor, ToolOutcome};
use ktu_tools::BashTool;

fn call(command: &str) -> ToolCall {
    ToolCall {
        id: ToolCallId::new("c"),
        name: "bash".to_owned(),
        arguments: serde_json::json!({ "command": command }).to_string(),
    }
}

#[tokio::test]
async fn runs_a_simple_command() {
    let tool = BashTool::new(".");
    let outcome = tool.execute(&call("echo ola")).await;
    assert!(!outcome.is_error);
    assert!(outcome.content.contains("ola"));
}

#[tokio::test]
async fn denies_destructive_commands() {
    let tool = BashTool::new(".");
    let outcome = tool.execute(&call("rm -rf /")).await;
    assert!(outcome.is_error);
    assert!(outcome.content.contains("recusado"));
}

#[tokio::test]
async fn non_zero_exit_is_an_error() {
    let tool = BashTool::new(".");
    let outcome = tool.execute(&call("exit 3")).await;
    assert!(outcome.is_error);
    assert!(outcome.content.contains("exit: 3"));
}

#[tokio::test]
async fn timeout_kills_the_command() {
    let tool = BashTool::new(".").with_timeout(Duration::from_millis(50));
    let outcome: ToolOutcome = tool.execute(&call("sleep 5")).await;
    assert!(outcome.is_error);
    assert!(outcome.content.contains("timeout"));
}
