//! Persistência do log da sessão em JSONL, pela porta [`Fs`].
//!
//! O log é a fonte da verdade (I3); retomar uma sessão é **carregar o log** e
//! continuar a acrescentar. A gravação é durável por *temp* + `rename` atómico.

use std::path::{Path, PathBuf};

use crate::ports::Fs;
use crate::session::{Event, SessionLog};
use crate::{Error, Result};

/// Caminho do ficheiro de uma sessão dentro de `dir`.
#[must_use]
pub fn session_path(dir: &Path, id: &str) -> PathBuf {
    dir.join(format!("{id}.jsonl"))
}

/// Serializa o log em JSONL (uma linha por evento).
///
/// # Errors
/// Devolve [`Error::Json`] se algum evento não serializar.
pub fn to_jsonl(log: &SessionLog) -> Result<String> {
    let mut out = String::new();
    for event in log.events() {
        let line = serde_json::to_string(event).map_err(|source| Error::json("evento", source))?;
        out.push_str(&line);
        out.push('\n');
    }
    Ok(out)
}

/// Reconstrói um log a partir de JSONL (ignora linhas vazias).
///
/// # Errors
/// Devolve [`Error::Json`] se alguma linha não desserializar.
pub fn from_jsonl(text: &str) -> Result<SessionLog> {
    let mut log = SessionLog::new();
    for line in text.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let event: Event =
            serde_json::from_str(line).map_err(|source| Error::json("linha", source))?;
        log.push(event);
    }
    Ok(log)
}

/// Grava o log de uma sessão de forma durável (*temp* + `rename`).
///
/// # Errors
/// Propaga erros de I/O da porta.
pub fn save(fs: &dyn Fs, dir: &Path, id: &str, log: &SessionLog) -> Result<()> {
    let path = session_path(dir, id);
    let temp = dir.join(format!("{id}.jsonl.tmp"));
    let text = to_jsonl(log)?;
    fs.write(&temp, text.as_bytes())?;
    fs.rename(&temp, &path)
}

/// Carrega o log de uma sessão, se existir.
///
/// # Errors
/// Propaga erros de I/O ou de JSON.
pub fn load(fs: &dyn Fs, dir: &Path, id: &str) -> Result<Option<SessionLog>> {
    let path = session_path(dir, id);
    if !fs.exists(&path) {
        return Ok(None);
    }
    let bytes = fs.read(&path)?;
    let text = String::from_utf8(bytes).map_err(|_| Error::io(&path, "conteudo nao e UTF-8"))?;
    from_jsonl(&text).map(Some)
}

/// Carrega o log tolerando uma **última linha parcial** (escrita interrompida por
/// *crash*): o prefixo válido é recuperado; qualquer outra linha malformada é erro.
///
/// # Errors
/// Propaga erros de I/O; linha malformada fora da última posição é erro de JSON.
pub fn load_recovering(fs: &dyn Fs, dir: &Path, id: &str) -> Result<Option<SessionLog>> {
    let path = session_path(dir, id);
    if !fs.exists(&path) {
        return Ok(None);
    }
    let bytes = fs.read(&path)?;
    let text = String::from_utf8(bytes).map_err(|_| Error::io(&path, "conteudo nao e UTF-8"))?;
    let lines: Vec<&str> = text
        .lines()
        .filter(|line| !line.trim().is_empty())
        .collect();
    let last = lines.len().saturating_sub(1);
    let mut log = SessionLog::new();
    for (index, line) in lines.iter().enumerate() {
        match serde_json::from_str::<Event>(line) {
            Ok(event) => log.push(event),
            Err(_) if index == last => break,
            Err(source) => return Err(Error::json("linha", source)),
        }
    }
    Ok(Some(log))
}

#[cfg(test)]
mod tests {
    use super::{from_jsonl, load, save, to_jsonl};
    use crate::domain::StopReason;
    use crate::ports::MemFs;
    use crate::session::{Event, SessionLog};
    use std::path::Path;

    fn sample_log() -> SessionLog {
        let mut log = SessionLog::new();
        log.push(Event::TurnStart {
            prompt: "ola".to_owned(),
        });
        log.push(Event::Assistant {
            content: "adeus".to_owned(),
            reasoning: "pensei".to_owned(),
            tool_calls: Vec::new(),
        });
        log.push(Event::TurnEnd {
            reason: StopReason::EndTurn,
        });
        log
    }

    #[test]
    fn jsonl_round_trips() {
        let log = sample_log();
        let text = to_jsonl(&log);
        assert!(text.is_ok());
        let text = text.unwrap_or_default();
        assert!(from_jsonl(&text).is_ok());
        assert_eq!(from_jsonl(&text).unwrap_or_default().events().len(), 3);
    }

    #[test]
    fn save_then_load_round_trips() {
        let fs = MemFs::new().with_dir("/.ktu/sessions");
        let log = sample_log();
        assert!(save(&fs, Path::new("/.ktu/sessions"), "s1", &log).is_ok());
        let loaded = load(&fs, Path::new("/.ktu/sessions"), "s1");
        assert!(loaded.is_ok());
        assert_eq!(
            loaded.unwrap_or_default().map(|log| log.events().len()),
            Some(3)
        );
    }

    #[test]
    fn load_missing_is_none() {
        let fs = MemFs::new();
        assert!(load(&fs, Path::new("/x"), "nope").is_ok());
        assert!(
            load(&fs, Path::new("/x"), "nope")
                .unwrap_or_default()
                .is_none()
        );
    }
}
