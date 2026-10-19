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
                    "bloco {position}: texto nao encontrado: {}{}",
                    excerpt(&edit.old),
                    nearby(&working, &edit.old)
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

/// Quantas linhas sugerir quando a âncora não é encontrada.
const NEARBY: usize = 3;

/// Distância máxima considerada (acima disto a linha não é "próxima" de nada).
const NEARBY_LIMIT: usize = 12;

/// Comprimento máximo de cada linha comparada/sugerida (caracteres).
const NEARBY_CHARS: usize = 80;

/// Sugere as linhas do corpo **mais próximas** da âncora que não foi encontrada.
///
/// Compara a primeira linha não vazia da âncora com cada linha do corpo (distância
/// de edição, limitada) e devolve as [`NEARBY`] melhores **em ordem de linha**
/// (determinístico, N1). Vazio quando não há nada suficientemente próximo.
fn nearby(body: &str, needle: &str) -> String {
    let Some(target) = needle
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .map(clip)
    else {
        return String::new();
    };
    let mut best: Vec<(usize, usize, String)> = Vec::new();
    let mut limit = NEARBY_LIMIT;
    for (index, line) in body.lines().enumerate() {
        let text = line.trim();
        if text.is_empty() {
            continue;
        }
        let text = clip(text);
        let Some(distance) = distance_within(&target, &text, limit) else {
            continue;
        };
        best.push((distance, index.saturating_add(1), text));
        best.sort_by_key(|entry| (entry.0, entry.1));
        best.truncate(NEARBY);
        if let Some((worst, _, _)) = best.last() {
            limit = *worst;
        }
    }
    if best.is_empty() {
        return String::new();
    }
    best.sort_by_key(|(_, line, _)| *line);
    let shown: Vec<String> = best
        .into_iter()
        .map(|(_, line, text)| format!("{line}: {text}"))
        .collect();
    format!("; mais proximas: {}", shown.join(" | "))
}

/// Trunca uma linha para comparação e para a mensagem.
fn clip(text: &str) -> String {
    if text.chars().count() <= NEARBY_CHARS {
        return text.to_owned();
    }
    let mut out: String = text.chars().take(NEARBY_CHARS).collect();
    out.push('…');
    out
}

/// Distância de edição (Levenshtein) **limitada**: `None` se exceder `limit`.
fn distance_within(left: &str, right: &str, limit: usize) -> Option<usize> {
    let left: Vec<char> = left.chars().collect();
    let right: Vec<char> = right.chars().collect();
    if left.len().abs_diff(right.len()) > limit {
        return None;
    }
    let mut previous: Vec<usize> = (0..=right.len()).collect();
    for (index, current_char) in left.iter().enumerate() {
        let mut row: Vec<usize> = Vec::with_capacity(right.len().saturating_add(1));
        row.push(index.saturating_add(1));
        let mut row_min = index.saturating_add(1);
        for (column, other) in right.iter().enumerate() {
            let substitute = previous
                .get(column)
                .copied()
                .unwrap_or(0)
                .saturating_add(usize::from(current_char != other));
            let delete = previous
                .get(column.saturating_add(1))
                .copied()
                .unwrap_or(0)
                .saturating_add(1);
            let insert = row.get(column).copied().unwrap_or(0).saturating_add(1);
            let value = substitute.min(delete).min(insert);
            row_min = row_min.min(value);
            row.push(value);
        }
        // Nenhuma célula da linha pode descer abaixo do mínimo: se ja passa do limite, corta.
        if row_min > limit {
            return None;
        }
        previous = row;
    }
    previous
        .get(right.len())
        .copied()
        .filter(|value| *value <= limit)
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
    fn a_missing_anchor_suggests_the_closest_lines_in_line_order() -> Result<(), String> {
        let body = "# Titulo\n\nfn soma(a: u32) -> u32 {\n    a + 1\n}\n\nfn dobro(a: u32) -> u32 {\n    a * 2\n}\n";
        let missing = edit("fn somma(a: u32) -> u32 {", "x");
        let Err(message) = apply_edits(body, &[missing]) else {
            return Err("esperava um erro de ancora".to_owned());
        };
        assert!(message.contains("texto nao encontrado"), "{message}");
        // A linha 3 e a mais proxima; vem primeiro (ordem de linha, N1).
        assert!(message.contains("3: fn soma(a: u32) -> u32 {"), "{message}");
        let first = message.find("3: ").unwrap_or(0);
        let second = message.find("7: ").unwrap_or(usize::MAX);
        assert!(first < second, "{message}");
        Ok(())
    }

    #[test]
    fn a_missing_anchor_far_from_everything_suggests_nothing() -> Result<(), String> {
        let body = "linha um\nlinha dois\n";
        let missing = edit("ZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZ", "x");
        let Err(message) = apply_edits(body, &[missing]) else {
            return Err("esperava um erro de ancora".to_owned());
        };
        assert!(!message.contains("mais proximas"), "{message}");
        Ok(())
    }

    #[test]
    fn an_ambiguous_anchor_does_not_suggest() -> Result<(), String> {
        let body = "igual\nigual\n";
        let Err(message) = apply_edits(body, &[edit("igual", "x")]) else {
            return Err("esperava um erro de ambiguidade".to_owned());
        };
        assert!(message.contains("ambiguo"), "{message}");
        assert!(!message.contains("mais proximas"), "{message}");
        Ok(())
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
