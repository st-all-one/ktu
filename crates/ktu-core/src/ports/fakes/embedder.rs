//! *Fakes* da porta [`Embedder`](super::super::Embedder).
//!
//! Nenhum teste do núcleo toca num servidor de embeddings: [`FakeEmbedder`] é
//! **determinístico** (hash, sem pesos nem rede) e [`UnavailableEmbedder`] força
//! a degradação graciosa (canal semântico desligado).

use super::super::{EmbedError, Embedder, EmbeddingMeta};

/// *Offset* da base FNV-1a (64 bits).
const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
/// Primo da base FNV-1a (64 bits).
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

/// Hash FNV-1a de 64 bits, com semente (a semente separa as dimensões).
fn fnv1a(seed: u64, bytes: &[u8]) -> u64 {
    let mut hash = seed;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

/// Converte `f64` em `f32` (o vetor é normalizado a seguir).
#[allow(
    clippy::as_conversions,
    clippy::cast_possible_truncation,
    reason = "vetor de embedding em f32; a perda de precisao do f64 e irrelevante"
)]
fn narrow(value: f64) -> f32 {
    value as f32
}

/// Vetor determinístico e **normalizado** para `text`.
///
/// Cada dimensão é `FNV-1a(texto ‖ índice)` mapeada para `[-1, 1]`; o vetor é
/// normalizado. O mesmo texto produz sempre o mesmo vetor, em qualquer máquina.
#[must_use]
fn hash_vector(text: &str, dimensions: usize) -> Vec<f32> {
    let dimensions = dimensions.max(1);
    let mut vector = Vec::with_capacity(dimensions);
    for index in 0..dimensions {
        let seed = FNV_OFFSET ^ u64::try_from(index).unwrap_or(u64::MAX);
        let bits = fnv1a(seed, text.as_bytes());
        let chunk = bits
            .to_le_bytes()
            .first_chunk::<4>()
            .copied()
            .unwrap_or([0; 4]);
        let value = f64::from(u32::from_le_bytes(chunk)) / f64::from(u32::MAX);
        vector.push(narrow(value.mul_add(2.0, -1.0)));
    }
    let norm = vector.iter().map(|value| value * value).sum::<f32>().sqrt();
    if norm > 0.0 {
        for value in &mut vector {
            *value /= norm;
        }
    }
    vector
}

/// Embedder **determinístico** por hash (testes, CI e uso offline).
#[derive(Clone, Debug)]
pub struct FakeEmbedder {
    meta: EmbeddingMeta,
}

impl FakeEmbedder {
    /// Dimensão por omissão do *fake* (barata e suficiente para ordenar).
    pub const DIMENSIONS: usize = 64;

    /// Cria o *fake* com a dimensão por omissão.
    #[must_use]
    pub fn new() -> Self {
        Self::with_dimensions(Self::DIMENSIONS)
    }

    /// Cria o *fake* com a dimensão indicada.
    #[must_use]
    pub fn with_dimensions(dimensions: usize) -> Self {
        Self {
            meta: EmbeddingMeta::new("fake", "hash-fnv1a", "1", dimensions),
        }
    }
}

impl Default for FakeEmbedder {
    fn default() -> Self {
        Self::new()
    }
}

impl Embedder for FakeEmbedder {
    fn meta(&self) -> &EmbeddingMeta {
        &self.meta
    }

    fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, EmbedError> {
        Ok(texts
            .iter()
            .map(|text| hash_vector(text, self.meta.dimensions))
            .collect())
    }
}

/// Embedder que **falha sempre**: prova a degradação graciosa (sem canal semântico).
#[derive(Clone, Debug)]
pub struct UnavailableEmbedder {
    meta: EmbeddingMeta,
}

impl UnavailableEmbedder {
    /// Cria o *fake* indisponível.
    #[must_use]
    pub fn new() -> Self {
        Self {
            meta: EmbeddingMeta::new("none", "none", "0", 1),
        }
    }
}

impl Default for UnavailableEmbedder {
    fn default() -> Self {
        Self::new()
    }
}

impl Embedder for UnavailableEmbedder {
    fn meta(&self) -> &EmbeddingMeta {
        &self.meta
    }

    fn embed(&self, _texts: &[String]) -> Result<Vec<Vec<f32>>, EmbedError> {
        Err(EmbedError::unavailable("sem provedor de embeddings"))
    }
}

#[cfg(test)]
mod tests {
    use super::{FakeEmbedder, UnavailableEmbedder, hash_vector};
    use crate::ports::{EmbedError, Embedder};

    #[test]
    fn hash_vector_is_deterministic_and_normalized() {
        let first = hash_vector("memoria do projecto", 32);
        let second = hash_vector("memoria do projecto", 32);
        assert_eq!(first, second, "o mesmo texto da o mesmo vetor");
        assert_eq!(first.len(), 32);
        let norm = first.iter().map(|value| value * value).sum::<f32>().sqrt();
        assert!((norm - 1.0).abs() < 1.0e-4, "norma {norm}");
    }

    #[test]
    fn hash_vector_separates_texts_and_dimensions() {
        assert_ne!(hash_vector("alfa", 16), hash_vector("beta", 16));
        assert_eq!(hash_vector("alfa", 16).len(), 16);
        assert_eq!(hash_vector("alfa", 8).len(), 8);
        assert_eq!(
            hash_vector("alfa", 0).len(),
            1,
            "dimensao 0 normaliza para 1"
        );
    }

    #[test]
    fn fake_embedder_preserves_order_and_identity() -> Result<(), EmbedError> {
        let embedder = FakeEmbedder::with_dimensions(8);
        assert_eq!(embedder.meta().provider, "fake");
        assert_eq!(embedder.meta().dimensions, 8);
        let texts = vec!["um".to_owned(), "dois".to_owned()];
        let vectors = embedder.embed(&texts)?;
        assert_eq!(vectors.len(), 2);
        assert_eq!(vectors.first(), Some(&hash_vector("um", 8)));
        assert_eq!(vectors.get(1), Some(&hash_vector("dois", 8)));
        Ok(())
    }

    #[test]
    fn unavailable_embedder_always_fails() {
        let embedder = UnavailableEmbedder::new();
        assert_eq!(embedder.meta().provider, "none");
        assert!(embedder.embed(&["x".to_owned()]).is_err());
    }
}
