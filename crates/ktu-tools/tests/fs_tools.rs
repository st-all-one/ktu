//! Integração das ferramentas de *filesystem* com um `MemFs` determinístico.

use ktu_core::domain::{ToolCall, ToolCallId};
use ktu_core::ports::MemFs;
use ktu_core::tools::{ToolExecutor, ToolOutcome};
use ktu_tools::{FileMutationQueue, FsTools};

fn call(name: &str, arguments: &str) -> ToolCall {
    ToolCall {
        id: ToolCallId::new("c"),
        name: name.to_owned(),
        arguments: arguments.to_owned(),
    }
}

fn mem() -> MemFs {
    MemFs::new()
        .with_file("/r/a.txt", "ola mundo")
        .with_file("/r/src/main.rs", "fn main() {}")
}

async fn run(tools: &FsTools<'_>, name: &str, arguments: &str) -> ToolOutcome {
    tools.execute(&call(name, arguments)).await
}

#[tokio::test]
async fn read_file_is_taint_wrapped() {
    let fs = mem();
    let mutations = FileMutationQueue::new();
    let tools = FsTools::new("/r", &fs, &mutations);
    let outcome = run(&tools, "read_file", r#"{"path":"a.txt"}"#).await;
    assert!(!outcome.is_error);
    assert!(outcome.content.contains("[untrusted: read_file]"));
    assert!(outcome.content.contains("ola mundo"));
}

#[tokio::test]
async fn write_then_edit_round_trips() {
    let fs = mem();
    let mutations = FileMutationQueue::new();
    let tools = FsTools::new("/r", &fs, &mutations);
    let written = run(
        &tools,
        "write_file",
        r#"{"path":"nota.txt","content":"linha um"}"#,
    )
    .await;
    assert!(!written.is_error);
    let edited = run(
        &tools,
        "edit_file",
        r#"{"path":"nota.txt","old":"um","new":"dois"}"#,
    )
    .await;
    assert!(!edited.is_error);
    let read = run(&tools, "read_file", r#"{"path":"nota.txt"}"#).await;
    assert!(read.content.contains("linha dois"));
}

#[tokio::test]
async fn edit_rejects_ambiguous_or_missing_text() {
    let fs = MemFs::new().with_file("/r/a.txt", "x x");
    let mutations = FileMutationQueue::new();
    let tools = FsTools::new("/r", &fs, &mutations);
    let ambiguous = run(
        &tools,
        "edit_file",
        r#"{"path":"a.txt","old":"x","new":"y"}"#,
    )
    .await;
    assert!(ambiguous.is_error);
    let missing = run(
        &tools,
        "edit_file",
        r#"{"path":"a.txt","old":"z","new":"y"}"#,
    )
    .await;
    assert!(missing.is_error);
}

#[tokio::test]
async fn rejects_path_escape() {
    let fs = mem();
    let mutations = FileMutationQueue::new();
    let tools = FsTools::new("/r", &fs, &mutations);
    let outcome = run(&tools, "read_file", r#"{"path":"../etc/passwd"}"#).await;
    assert!(outcome.is_error);
}

#[tokio::test]
async fn glob_and_grep_walk_the_root() {
    let fs = mem();
    let mutations = FileMutationQueue::new();
    let tools = FsTools::new("/r", &fs, &mutations);
    let globbed = run(&tools, "glob", r#"{"pattern":"src/**/*.rs"}"#).await;
    assert!(globbed.content.contains("src/main.rs"));
    let grepped = run(&tools, "grep", r#"{"pattern":"main"}"#).await;
    assert!(grepped.content.contains("src/main.rs"));
}
