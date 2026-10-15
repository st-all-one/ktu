//! Integração do *trash*: mover, listar e restaurar.

use ktu_core::domain::{ToolCall, ToolCallId};
use ktu_core::ports::{FixedClock, Fs, MemFs, Timestamp};
use ktu_core::tools::ToolExecutor;
use ktu_tools::trash::{TrashTool, list, restore};

fn call(path: &str) -> ToolCall {
    ToolCall {
        id: ToolCallId::new("c"),
        name: "trash_file".to_owned(),
        arguments: serde_json::json!({ "path": path }).to_string(),
    }
}

#[tokio::test]
async fn trash_then_restore_round_trips() {
    let fs = MemFs::new()
        .with_file("/r/a.txt", "ola")
        .with_dir("/r/.ktu/trash");
    let clock = FixedClock::new(Timestamp::from_millis(1_000));
    let tool = TrashTool::new("/r", &fs, &clock);

    let outcome = tool.execute(&call("a.txt")).await;
    assert!(!outcome.is_error);
    assert!(!fs.exists(std::path::Path::new("/r/a.txt")));
    let entries = list(&fs, std::path::Path::new("/r")).unwrap_or_default();
    assert_eq!(entries.len(), 1);

    let restored = restore(&fs, std::path::Path::new("/r"), "1000-a.txt");
    assert!(restored.is_ok());
    assert!(fs.exists(std::path::Path::new("/r/a.txt")));
}
