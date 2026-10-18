//! Texto para edição: BOM, *line-endings* e edições multi-bloco **atómicas**.
//!
//! Lógica **pura** (sem I/O): o `edit` detecta o BOM/ending do ficheiro, calcula
//! a mudança em LF e **restaura** o original na gravação. Editar um ficheiro CRLF
//! num ambiente Unix não pode mudar todas as linhas (diff falso / corrupção).

/// Marca de ordem de bytes (U+FEFF) no início do ficheiro.
const BOM: char = '\u{feff}';

/// *Line-ending* original do ficheiro.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Ending {
    /// `\n` (Unix).
    Lf,
    /// `\r\n` (Windows).
    Crlf,
}

/// Documento de texto com o BOM e o *line-ending* **originais**.
#[derive(Debug, Clone)]
pub struct Document {
    bom: bool,
    ending: Ending,
    body: String,
}

impl Document {
    /// Detecta o BOM e o *line-ending* do conteúdo em bruto.
    #[must_use]
    pub fn parse(bytes: &[u8]) -> Self {
        let text = String::from_utf8_lossy(bytes).into_owned();
        let (bom, rest) = match text.strip_prefix(BOM) {
            Some(rest) => (true, rest.to_owned()),
            None => (false, text),
        };
        let ending = if rest.contains("\r\n") {
            Ending::Crlf
        } else {
            Ending::Lf
        };
        let body = to_lf(&rest);
        Self { bom, ending, body }
    }

    /// Corpo normalizado (sem BOM, com `\n`).
    #[must_use]
    pub fn body(&self) -> &str {
        &self.body
    }

    /// Recompõe os bytes com o BOM e o *line-ending* originais.
    #[must_use]
    pub fn render(&self, body: &str) -> Vec<u8> {
        let mut text = if self.ending == Ending::Crlf {
            body.replace('\n', "\r\n")
        } else {
            body.to_owned()
        };
        if self.bom {
            text.insert(0, BOM);
        }
        text.into_bytes()
    }
}

/// Normaliza um fragmento para LF (o ficheiro é comparado em LF).
#[must_use]
pub fn to_lf(text: &str) -> String {
    text.replace("\r\n", "\n")
}

/// Uma edição: substitui a **única** ocorrência de `old` por `new`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Edit {
    /// Texto âncora (tem de ocorrer exactamente uma vez).
    pub old: String,
    /// Texto de substituição.
    pub new: String,
}

/// Aplica as edições **atomicamente**: erro se alguma âncora não for única.
///
/// Nada é gravado quando uma edição falha — o resultado só é devolvido no fim.
///
/// # Errors
/// Devolve mensagem com o índice do bloco (1-based) que falhou.
pub fn apply_edits(body: &str, edits: &[Edit]) -> Result<String, String> {
    let mut working = body.to_owned();
    for (index, edit) in edits.iter().enumerate() {
        let position = index.saturating_add(1);
        match working.matches(edit.old.as_str()).count() {
            0 => {
                return Err(format!(
                    "bloco {position}: texto nao encontrado: {}",
                    excerpt(&edit.old)
                ));
            }
            1 => {
                working = working.replacen(edit.old.as_str(), edit.new.as_str(), 1);
            }
            count => {
                return Err(format!(
                    "bloco {position}: texto ambiguo ({count} ocorrencias)"
                ));
            }
        }
    }
    Ok(working)
}

/// Primeira linha de `text`, truncada, para mensagens de erro legíveis.
fn excerpt(text: &str) -> String {
    let line = text.lines().next().unwrap_or("").trim();
    if line.chars().count() <= 60 {
        return line.to_owned();
    }
    let mut out: String = line.chars().take(60).collect();
    out.push('…');
    out
}

#[cfg(test)]
mod tests {
    use super::{Document, Edit, apply_edits, to_lf};

    fn edit(old: &str, new: &str) -> Edit {
        Edit {
            old: old.to_owned(),
            new: new.to_owned(),
        }
    }

    #[test]
    fn preserves_crlf_and_bom() {
        let bytes = "\u{feff}um\r\ndois\r\n".as_bytes();
        let document = Document::parse(bytes);
        assert_eq!(document.body(), "um\ndois\n");
        let rendered = document.render("um\nDOIS\n");
        assert_eq!(String::from_utf8_lossy(&rendered), "\u{feff}um\r\nDOIS\r\n");
    }

    #[test]
    fn lf_stays_lf_without_bom() {
        let document = Document::parse(b"a\nb\n");
        assert_eq!(document.body(), "a\nb\n");
        assert_eq!(
            String::from_utf8_lossy(&document.render("a\nb\n")),
            "a\nb\n"
        );
    }

    #[test]
    fn applies_all_edits_in_one_pass() {
        let edits = [edit("a", "A"), edit("b", "B")];
        assert_eq!(apply_edits("a b", &edits).unwrap_or_default(), "A B");
    }

    #[test]
    fn is_atomic_when_an_anchor_is_ambiguous() {
        let edits = [edit("a", "A"), edit("x", "X")];
        assert!(apply_edits("a a", &edits).is_err());
    }

    #[test]
    fn reports_the_missing_block() {
        let edits = [edit("a", "A"), edit("z", "Z")];
        let error = apply_edits("a b", &edits).err().unwrap_or_default();
        assert!(error.contains("bloco 2"), "{error}");
    }

    #[test]
    fn normalizes_model_fragments_to_lf() {
        assert_eq!(to_lf("a\r\nb"), "a\nb");
    }
}
