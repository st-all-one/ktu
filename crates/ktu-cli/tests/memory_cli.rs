//! Integração dos comandos `ktu memory export|list` e `ktu doctor --memory`.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use ktu_core::memory::{Memory, MemoryDraft, MemoryKind};
use ktu_core::ports::{FixedClock, Timestamp};
use ktu_memory::JsonlMemory;
use ktu_tools::StdFs;

fn seed(root: &Path) {
    let fs = StdFs;
    let clock = FixedClock::new(Timestamp::from_millis(1));
    let memory = JsonlMemory::new(&fs, &clock, root, root.join(".agents").join("memory"));
    let _written = memory.write(&MemoryDraft::new(
        MemoryKind::Fact,
        "o ktu memoriza entre sessoes",
    ));
}

fn temp_root(tag: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("ktu-cli-{tag}-{}", std::process::id()));
    let _ignored = std::fs::create_dir_all(root.join(".agents").join("memory"));
    root
}

fn run(args: &[&str], root: &Path) -> Option<Output> {
    Command::new(env!("CARGO_BIN_EXE_ktu"))
        .args(args)
        .arg(root)
        .output()
        .ok()
}

#[test]
fn memory_export_prints_the_statement() {
    let root = temp_root("export");
    seed(&root);
    let Some(output) = run(&["memory", "export", "--root"], &root) else {
        return;
    };
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success(), "exit: {output:?}");
    assert!(stdout.contains("o ktu memoriza entre sessoes"));
}

#[test]
fn memory_list_and_doctor_are_healthy() {
    let root = temp_root("list");
    seed(&root);
    let Some(listed) = run(&["memory", "list", "--root"], &root) else {
        return;
    };
    assert!(String::from_utf8_lossy(&listed.stdout).contains("o ktu memoriza entre sessoes"));

    let Some(doctor) = run(&["doctor", "--memory", "--root"], &root) else {
        return;
    };
    assert!(
        doctor.status.success(),
        "doctor: {}",
        String::from_utf8_lossy(&doctor.stdout)
    );
    assert!(String::from_utf8_lossy(&doctor.stdout).contains("memory.notes: 1"));
}
