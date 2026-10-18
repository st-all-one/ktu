//! Índice vetorial **derivado** e o canal semântico do recall (H8).
//!
//! O índice vive em `<memoria>/.idx/embeddings.jsonl` e é **derivado** de
//! `notes.jsonl` + provedor: reconstruível e descartável. A primeira linha é o
//! cabeçalho de identidade — o **mesmo codec do knudge**, pelo que as duas vistas
//! nunca divergem. Se a identidade não casar com a configuração, o índice é
//! **invalidado** (re-embed completo).
//!
//! Degradação graciosa: sem provedor ou com o índice ausente/ilegível, o recall é
//! lexical e **desliga sem aviso** (um índice malformado é tratado como ausente —
//! é 100 % derivado e o próximo dreno reescreve-o).

use std::path::{Path, PathBuf};

use knudge_core::embeddings::rank_query;
use knudge_core::embeddings::{EmbeddingIndex, EmbeddingMeta as EngineMeta, Prefix, Similarity};
use ktu_core::memory::{MemoryError, MemoryResult};
use ktu_core::memory_config::Embeddings;
use ktu_core::ports::{EmbedError, Embedder, Fs};

use crate::convert::{map_core, map_knudge};

/// Directório do derivado dentro da memória.
pub const INDEX_DIR: &str = ".idx";
/// Nome do ficheiro do índice vetorial (igual ao do knudge).
pub const INDEX_FILE: &str = "embeddings.jsonl";
/// Ids do canal semântico que entram na fusão RRF.
pub const SEMANTIC_TOP_K: usize = 20;
/// Similaridade mínima para entrar no canal semântico (cosseno normalizado).
pub const MIN_SCORE: f64 = 0.0;

/// Caminho do índice vetorial para um directório de memória.
#[must_use]
pub fn index_path(memory_dir: &Path) -> PathBuf {
    memory_dir.join(INDEX_DIR).join(INDEX_FILE)
}

/// Directório do derivado (para a borda o remover em `--force`).
#[must_use]
pub fn index_dir(memory_dir: &Path) -> PathBuf {
    memory_dir.join(INDEX_DIR)
}

/// Identidade do motor a partir da configuração (mesmo cabeçalho do knudge).
///
/// A métrica é **cosseno** (a dos modelos `-cos`, como o `embeddinggemma`); o
/// prefixo é resolvido de `embeddings.prefix` contra o nome do modelo.
///
/// # Errors
/// Devolve erro de schema se o prefixo configurado for desconhecido.
pub fn engine_meta(config: &Embeddings) -> MemoryResult<EngineMeta> {
    let prefix =
        Prefix::from_config(&config.prefix, &config.model).map_err(|error| map_knudge(&error))?;
    EngineMeta::new(
        config.provider.as_str(),
        config.model.clone(),
        config.revision.clone(),
        config.dimensions,
        Similarity::Cosine,
        prefix,
    )
    .map_err(|error| map_knudge(&error))
}

/// Carrega o índice, devolvendo `None` se estiver ausente, vazio, **ilegível** ou
/// com identidade diferente da esperada (invalidação — tudo é re-embed).
///
/// # Errors
/// Propaga apenas erros de I/O da porta.
pub fn load(
    fs: &dyn Fs,
    memory_dir: &Path,
    expected: &EngineMeta,
) -> MemoryResult<Option<EmbeddingIndex>> {
    let path = index_path(memory_dir);
    if !fs.exists(&path) {
        return Ok(None);
    }
    let bytes = fs.read(&path).map_err(map_core)?;
    let Ok(text) = String::from_utf8(bytes) else {
        return Ok(None);
    };
    if text.trim().is_empty() {
        return Ok(None);
    }
    Ok(EmbeddingIndex::parse(&text, expected).ok().flatten())
}

/// Grava o índice de forma durável (`temp → fsync → rename`).
///
/// # Errors
/// Propaga erros de I/O e de serialização.
pub fn save(fs: &dyn Fs, memory_dir: &Path, index: &EmbeddingIndex) -> MemoryResult<()> {
    let path = index_path(memory_dir);
    let text = index.serialize().map_err(|error| map_knudge(&error))?;
    let temp = path.with_extension("jsonl.tmp");
    fs.write(&temp, text.as_bytes()).map_err(map_core)?;
    fs.sync(&temp).map_err(map_core)?;
    fs.rename(&temp, &path).map_err(map_core)?;
    Ok(())
}

/// Ids do canal semântico para `query` (embute a consulta e ordena por similaridade).
///
/// # Errors
/// Propaga a indisponibilidade do provedor (`MemoryError::Unavailable`).
pub fn semantic_ids(
    index: &EmbeddingIndex,
    embedder: &dyn Embedder,
    query: &str,
    top_k: usize,
) -> MemoryResult<Vec<String>> {
    if index.is_empty() {
        return Ok(Vec::new());
    }
    let text = index.meta.prefix.query(query);
    let vectors = embedder.embed(&[text]).map_err(map_embed)?;
    let Some(vector) = vectors.first() else {
        return Ok(Vec::new());
    };
    Ok(rank_query(index, vector, top_k, MIN_SCORE))
}

/// Traduz o erro da porta de embeddings.
pub(crate) fn map_embed(error: EmbedError) -> MemoryError {
    match error {
        EmbedError::Unavailable(message) => MemoryError::unavailable(message),
        EmbedError::Invalid(message) => MemoryError::schema(message),
        other => MemoryError::schema(other.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use ktu_core::memory_config::{EmbeddingProvider, Embeddings};
    use ktu_core::ports::{Embedder, FakeEmbedder, Fs, MemFs};

    use super::{INDEX_FILE, engine_meta, index_path, load, save, semantic_ids};
    use knudge_core::embeddings::EmbeddingIndex;

    fn config(provider: EmbeddingProvider) -> Embeddings {
        Embeddings {
            provider,
            dimensions: 16,
            ..Embeddings::default()
        }
    }

    /// Configuração **sem prefixo**: o texto do documento e o da consulta coincidem.
    fn plain_config() -> Embeddings {
        Embeddings {
            prefix: "none".to_owned(),
            ..config(EmbeddingProvider::None)
        }
    }

    #[test]
    fn identity_follows_the_configuration() -> Result<(), String> {
        let meta = engine_meta(&config(EmbeddingProvider::Http)).map_err(|e| e.to_string())?;
        assert_eq!(meta.provider, "http");
        assert_eq!(meta.model, "google/embeddinggemma-2");
        assert_eq!(meta.dimensions, 16);
        assert_eq!(meta.prefix.as_str(), "embeddinggemma");
        let mut off = config(EmbeddingProvider::None);
        off.prefix = "nope".to_owned();
        assert!(engine_meta(&off).is_err());
        Ok(())
    }

    #[test]
    fn save_then_load_round_trips() -> Result<(), String> {
        let fs = MemFs::new();
        let dir = std::path::Path::new("/mem");
        let meta = engine_meta(&config(EmbeddingProvider::Http)).map_err(|e| e.to_string())?;
        assert!(load(&fs, dir, &meta).map_err(|e| e.to_string())?.is_none());

        let mut index = EmbeddingIndex::new(meta.clone());
        index.insert("fact_1", "hash1", vec![1.0, 0.0]);
        save(&fs, dir, &index).map_err(|e| e.to_string())?;
        assert!(fs.exists(&index_path(dir)));
        let loaded = load(&fs, dir, &meta).map_err(|e| e.to_string())?;
        let loaded = loaded.ok_or("indice devia carregar")?;
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded.body_hash("fact_1"), Some("hash1"));
        Ok(())
    }

    #[test]
    fn identity_mismatch_and_garbage_invalidate() -> Result<(), String> {
        let fs = MemFs::new();
        let dir = std::path::Path::new("/mem");
        let meta = engine_meta(&config(EmbeddingProvider::Http)).map_err(|e| e.to_string())?;
        let mut index = EmbeddingIndex::new(meta.clone());
        index.insert("fact_1", "hash1", vec![1.0]);
        save(&fs, dir, &index).map_err(|e| e.to_string())?;

        let mut other = config(EmbeddingProvider::Http);
        other.model = "outro/modelo".to_owned();
        let other = engine_meta(&other).map_err(|e| e.to_string())?;
        assert!(
            load(&fs, dir, &other).map_err(|e| e.to_string())?.is_none(),
            "modelo diferente invalida o indice"
        );

        let garbage = MemFs::new().with_file(index_path(dir), b"{ nao e json }".to_vec());
        assert!(
            load(&garbage, dir, &meta)
                .map_err(|e| e.to_string())?
                .is_none(),
            "indice ilegivel e tratado como ausente"
        );
        assert_eq!(INDEX_FILE, "embeddings.jsonl");
        Ok(())
    }

    #[test]
    fn semantic_ids_ranks_by_similarity() -> Result<(), String> {
        let embedder = FakeEmbedder::with_dimensions(8);
        let meta = engine_meta(&plain_config()).map_err(|e| e.to_string())?;
        let mut index = EmbeddingIndex::new(meta);
        for (id, text) in [("fact_a", "memoria do projecto"), ("fact_b", "outra coisa")] {
            let vectors = embedder
                .embed(&[text.to_owned()])
                .map_err(|e| e.to_string())?;
            let vector = vectors.into_iter().next().ok_or("sem vetor")?;
            index.insert(id, "h", vector);
        }
        let ids =
            semantic_ids(&index, &embedder, "memoria do projecto", 5).map_err(|e| e.to_string())?;
        assert_eq!(ids.first().map(String::as_str), Some("fact_a"));
        assert_eq!(ids.len(), 2);

        let empty = EmbeddingIndex::new(index.meta.clone());
        let none = semantic_ids(&empty, &embedder, "x", 5).map_err(|e| e.to_string())?;
        assert_eq!(none.len(), 0);
        Ok(())
    }
}
