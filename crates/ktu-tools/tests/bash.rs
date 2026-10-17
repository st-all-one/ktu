//! Integração da tool `bash`: confinamento de caminhos ao *root* (`risk_01ua5ei2`).

use ktu_core::domain::{ToolCall, ToolCallId};
use ktu_core::tools::ToolExecutor;
use ktu_tools::BashTool;

fn call(command: &str) -> ToolCall {
    ToolCall {
        id: ToolCallId::new("1"),
        name: "bash".to_owned(),
        arguments: serde_json::json!({ "command": command }).to_string(),
    }
}

#[tokio::test]
async fn bash_denies_writes_outside_the_root() -> Result<(), String> {
    let root = std::env::temp_dir().join(format!("ktu-bash-{}", std::process::id()));
    std::fs::create_dir_all(&root).map_err(|error| error.to_string())?;
    let tool = BashTool::new(&root);

    let escaped = tool.execute(&call("echo x > /tmp/ktu-escape-out")).await;
    assert!(escaped.is_error, "escrita fora do root devia ser recusada");
    assert!(escaped.content.contains("recusado"));

    let home = tool.execute(&call("cat ~/.bashrc")).await;
    assert!(home.is_error, "expansao de home devia ser recusada");

    let inside = tool.execute(&call("echo oi")).await;
    assert!(!inside.is_error, "comando dentro do root devia correr");

    let _ignored = std::fs::remove_dir_all(&root);
    Ok(())
}
