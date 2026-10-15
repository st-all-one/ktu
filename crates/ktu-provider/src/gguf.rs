//! Leitura mínima do cabeçalho GGUF (magic, versão e metadados-chave).
//!
//! Usado pelo `providers doctor` para validar que o ficheiro é um GGUF e
//! reportar `general.architecture` e `general.file_type`. Não carrega tensores.

use std::path::Path;

use ktu_core::provider::ProviderError;

/// Metadados essenciais de um ficheiro GGUF.
#[derive(Clone, Debug)]
pub struct GgufMeta {
    /// Versão do formato GGUF.
    pub version: u32,
    /// Número de tensores declarado.
    pub tensor_count: u64,
    /// Arquitectura declarada (`general.architecture`).
    pub architecture: Option<String>,
    /// Tipo de quantização (`general.file_type`).
    pub file_type: Option<u32>,
}

/// Lê os metadados essenciais de um GGUF.
///
/// # Errors
/// Devolve [`ProviderError`] se o ficheiro não for legível ou não for um GGUF
/// válido (magic, versão ou metadados malformados).
pub fn read_metadata(path: &Path) -> Result<GgufMeta, ProviderError> {
    let bytes = std::fs::read(path)
        .map_err(|error| ProviderError::Unavailable(format!("{}: {error}", path.display())))?;
    parse(&bytes).map_err(|reason| {
        ProviderError::Invalid(format!("gguf invalido em {}: {reason}", path.display()))
    })
}

fn parse(bytes: &[u8]) -> Result<GgufMeta, &'static str> {
    let mut cursor = Cursor::new(bytes);
    let magic = cursor.take(4).ok_or("truncado")?;
    if magic != b"GGUF" {
        return Err("magic != GGUF");
    }
    let version = cursor.u32().ok_or("versao")?;
    let tensor_count = cursor.u64().ok_or("tensor_count")?;
    let kv_count = cursor.u64().ok_or("kv_count")?;
    let mut architecture = None;
    let mut file_type = None;
    let limit = kv_count.min(100_000);
    for _ in 0..limit {
        let Some(key) = cursor.string() else {
            break;
        };
        let Some(kind) = cursor.u32() else {
            break;
        };
        match key.as_str() {
            "general.architecture" => architecture = read_string(&mut cursor, kind),
            "general.file_type" => file_type = read_u32(&mut cursor, kind),
            _ => skip(&mut cursor, kind),
        }
    }
    Ok(GgufMeta {
        version,
        tensor_count,
        architecture,
        file_type,
    })
}

fn read_string(cursor: &mut Cursor<'_>, kind: u32) -> Option<String> {
    if kind == 8 {
        cursor.string()
    } else {
        skip(cursor, kind);
        None
    }
}

fn read_u32(cursor: &mut Cursor<'_>, kind: u32) -> Option<u32> {
    if kind == 4 {
        cursor.u32()
    } else {
        skip(cursor, kind);
        None
    }
}

/// Salta um valor do tipo GGUF `kind`.
fn skip(cursor: &mut Cursor<'_>, kind: u32) {
    match kind {
        0 | 1 | 7 => cursor.skip_bytes(1),
        2 | 3 => cursor.skip_bytes(2),
        4..=6 => cursor.skip_bytes(4),
        10..=12 => cursor.skip_bytes(8),
        8 => {
            if let Some(length) = cursor.u64() {
                let length = usize::try_from(length).unwrap_or(usize::MAX);
                cursor.skip_bytes(length);
            }
        }
        9 => {
            let element = cursor.u32().unwrap_or(0);
            let count = cursor.u64().unwrap_or(0);
            let count = count.min(1_000_000);
            for _ in 0..count {
                skip(cursor, element);
            }
        }
        _ => cursor.pos = cursor.bytes.len(),
    }
}

struct Cursor<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> Cursor<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, pos: 0 }
    }

    fn take(&mut self, length: usize) -> Option<&'a [u8]> {
        let end = self.pos.checked_add(length)?;
        let slice = self.bytes.get(self.pos..end)?;
        self.pos = end;
        Some(slice)
    }

    fn skip_bytes(&mut self, length: usize) {
        let _ignored: Option<&[u8]> = self.take(length);
    }

    fn u32(&mut self) -> Option<u32> {
        let bytes: [u8; 4] = self.take(4)?.try_into().ok()?;
        Some(u32::from_le_bytes(bytes))
    }

    fn u64(&mut self) -> Option<u64> {
        let bytes: [u8; 8] = self.take(8)?.try_into().ok()?;
        Some(u64::from_le_bytes(bytes))
    }

    fn string(&mut self) -> Option<String> {
        let length = self.u64()?;
        let length = usize::try_from(length).ok()?;
        let bytes = self.take(length)?;
        Some(String::from_utf8_lossy(bytes).into_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::parse;

    fn gguf_with_file_type() -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(b"GGUF");
        bytes.extend_from_slice(&3_u32.to_le_bytes());
        bytes.extend_from_slice(&0_u64.to_le_bytes()); // tensor_count
        bytes.extend_from_slice(&1_u64.to_le_bytes()); // kv_count
        let key = b"general.file_type";
        let key_len = u64::try_from(key.len()).unwrap_or(0);
        bytes.extend_from_slice(&key_len.to_le_bytes());
        bytes.extend_from_slice(key);
        bytes.extend_from_slice(&4_u32.to_le_bytes()); // uint32
        bytes.extend_from_slice(&42_u32.to_le_bytes());
        bytes
    }

    #[test]
    fn reads_file_type() {
        let meta = parse(&gguf_with_file_type());
        assert_eq!(meta.map(|meta| meta.file_type), Ok(Some(42)));
    }

    #[test]
    fn rejects_non_gguf() {
        assert!(parse(b"nope").is_err());
    }
}
