//! `ktu doctor` — diagnostica a **estrutura do projecto**: o que o `ktu init` cria.
//!
//! Responde a «este projecto está pronto para o ktu?» com itens observáveis, cada um
//! com o **remédio** quando não está (uma recusa sem remédio não é accionável). Só
//! olha para a **existência** do que o `ktu init` gera — o *conteúdo* tem porta
//! própria por área (`ktu config doctor`, `ktu memory doctor`, `ktu providers doctor`).
//!
//! O `AGENTS.md` **não** é diagnosticado: é o ficheiro de instruções do projecto,
//! escrito pelo humano — o ktu não escreve nele (ADR 0021) e portanto não tem nada a
//! verificar ali. O protocolo do agente vive em `ktu prime` e na base do prompt.
//!
//! As outras áreas têm porta própria — este comando **aponta-as**, não as duplica:
//! `ktu config doctor` (schema), `ktu memory doctor` (memória), `ktu providers doctor`
//! (provider).

use std::path::Path;
use std::process::ExitCode;

use ktu_core::layout;

use crate::output::{self, Format};

/// Estado de um item (enum: um `bool` não diz o que falhou nem o que fazer).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State {
    /// Em dia.
    Ok,
    /// Ausente ou desactualizado — o projecto não está pronto.
    Pending,
}

/// Um item do diagnóstico.
struct Item {
    /// Nome curto do item.
    name: &'static str,
    /// Estado.
    state: State,
    /// O que se observou.
    detail: String,
    /// O que fazer, quando não está em dia.
    remedy: Option<String>,
}

impl Item {
    /// Item em dia.
    fn ok(name: &'static str, detail: String) -> Self {
        Self {
            name,
            state: State::Ok,
            detail,
            remedy: None,
        }
    }

    /// Item por resolver (com remédio).
    fn pending(name: &'static str, detail: String, remedy: String) -> Self {
        Self {
            name,
            state: State::Pending,
            detail,
            remedy: Some(remedy),
        }
    }

    /// Linha de texto humano.
    fn render(&self) -> String {
        let mark = match self.state {
            State::Ok => "ok",
            State::Pending => "FALTA",
        };
        match &self.remedy {
            Some(remedy) => format!("{}: {mark} ({}) — {remedy}", self.name, self.detail),
            None => format!("{}: {mark} ({})", self.name, self.detail),
        }
    }

    /// Representação máquina-legível.
    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "name": self.name,
            "ok": self.state == State::Ok,
            "detail": self.detail,
            "remedy": self.remedy,
        })
    }
}

/// As outras portas de diagnóstico (uma só por área).
const OTHERS: &str = "ktu config doctor · ktu memory doctor · ktu providers doctor";

/// Diagnostica o projecto e imprime o relatório (`stdout` = dados).
pub(crate) fn run(root: &Path, format: Format) -> ExitCode {
    let items = [
        layout_item(root),
        memory_source_item(root),
        config_item(root),
    ];
    let healthy = items.iter().all(|item| item.state == State::Ok);
    let mut lines: Vec<String> = items.iter().map(Item::render).collect();
    lines.push(format!("outros: {OTHERS}"));
    let data: Vec<serde_json::Value> = items.iter().map(Item::json).collect();
    output::emit(
        format,
        "doctor",
        &lines.join("\n"),
        &serde_json::json!({ "healthy": healthy, "items": data, "others": OTHERS }),
    );
    if healthy {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

/// Layout: os directórios do `ktu init` que existem.
fn layout_item(root: &Path) -> Item {
    let missing: Vec<&str> = layout::INIT_DIRS
        .iter()
        .copied()
        .filter(|dir| !root.join(dir).is_dir())
        .collect();
    let total = layout::INIT_DIRS.len();
    let present = total.saturating_sub(missing.len());
    if missing.is_empty() {
        return Item::ok("layout", format!("{present}/{total}"));
    }
    Item::pending(
        "layout",
        format!("{present}/{total} (falta {})", missing.join(", ")),
        "corre `ktu init`".to_owned(),
    )
}

/// Fonte da verdade da memória (`.agents/memory/notes.jsonl`): existe.
fn memory_source_item(root: &Path) -> Item {
    if root.join(layout::NOTES_FILE).is_file() {
        return Item::ok("memory_source", layout::NOTES_FILE.to_owned());
    }
    Item::pending(
        "memory_source",
        format!("{} ausente", layout::NOTES_FILE),
        "corre `ktu init`".to_owned(),
    )
}

/// Config do projecto (`.agents/config.toml`): existe.
fn config_item(root: &Path) -> Item {
    if root.join(layout::CONFIG_FILE).is_file() {
        return Item::ok("config", layout::CONFIG_FILE.to_owned());
    }
    Item::pending(
        "config",
        format!("{} ausente", layout::CONFIG_FILE),
        "corre `ktu init`".to_owned(),
    )
}

#[cfg(test)]
mod tests {
    use super::{State, config_item, layout_item, memory_source_item};
    use ktu_core::layout;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicU64, Ordering};

    static COUNTER: AtomicU64 = AtomicU64::new(0);

    /// Raiz temporária única por teste (criada).
    fn fresh_root(tag: &str) -> PathBuf {
        let id = COUNTER.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!("ktu-doc-{tag}-{}-{id}", std::process::id()));
        let _ignored = fs::remove_dir_all(&root);
        let _ignored = fs::create_dir_all(&root);
        root
    }

    /// Escreve um ficheiro relativo à raiz, criando os directórios.
    fn write_at(root: &Path, relative: &str, content: &str) -> bool {
        let path = root.join(relative);
        if let Some(parent) = path.parent()
            && fs::create_dir_all(parent).is_err()
        {
            return false;
        }
        fs::write(&path, content).is_ok()
    }

    #[test]
    fn a_fresh_directory_is_missing_everything() {
        let root = fresh_root("vazio");
        assert_eq!(layout_item(&root).state, State::Pending);
        assert_eq!(memory_source_item(&root).state, State::Pending);
        assert_eq!(config_item(&root).state, State::Pending);
    }

    #[test]
    fn the_generated_layout_is_healthy() {
        let root = fresh_root("layout");
        for dir in layout::INIT_DIRS {
            assert!(fs::create_dir_all(root.join(dir)).is_ok());
        }
        assert!(write_at(&root, layout::NOTES_FILE, ""));
        assert!(write_at(&root, layout::CONFIG_FILE, ""));
        assert_eq!(layout_item(&root).state, State::Ok);
        assert_eq!(memory_source_item(&root).state, State::Ok);
        assert_eq!(config_item(&root).state, State::Ok);
    }
}
