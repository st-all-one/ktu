//! Nome automático de sessão (heurística determinística, sem LLM).
//!
//! Listar sessões por `id` é pouco legível; o título curto derivado do **primeiro**
//! pedido resolve isso sem custo nem não-determinismo. É **dado derivado** (o log
//! continua a ser a verdade): pode ser recalculado a qualquer momento.

use super::{Event, SessionLog};

/// Máximo de palavras do título.
pub const MAX_WORDS: usize = 8;

/// Máximo de caracteres do título.
pub const MAX_CHARS: usize = 60;

/// Deriva um título curto de um pedido (≤ [`MAX_WORDS`] palavras e [`MAX_CHARS`]).
///
/// Puro e determinístico: remove *tags* XML, cercas de código e pontuação de
/// extremidade, colapsa espaços e corta na fronteira de palavra. Devolve `None`
/// quando não sobra nada utilizável (o consumidor usa o `id` como *fallback*).
#[must_use]
pub fn title_from(prompt: &str) -> Option<String> {
    let cleaned = clean(prompt);
    let words: Vec<&str> = cleaned.split_whitespace().take(MAX_WORDS).collect();
    let joined = words.join(" ");
    if joined.is_empty() {
        return None;
    }
    Some(truncate_chars(&joined, MAX_CHARS))
}

/// Título de uma sessão: o do **primeiro** pedido registado no log.
#[must_use]
pub fn title_of(log: &SessionLog) -> Option<String> {
    log.events().iter().find_map(|event| match event {
        Event::TurnStart { prompt } => title_from(prompt),
        _ => None,
    })
}

/// Remove ruído do pedido: *tags*, cercas de código, blocos e pontuação de extremidade.
fn clean(prompt: &str) -> String {
    let mut out = String::new();
    let mut in_tag = false;
    let mut in_fence = false;
    for line in prompt.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("```") {
            in_fence = !in_fence;
            continue;
        }
        if in_fence {
            continue;
        }
        for ch in line.chars() {
            match ch {
                '<' => in_tag = true,
                '>' => in_tag = false,
                _ if in_tag => {}
                _ => out.push(ch),
            }
        }
        out.push(' ');
    }
    trim_punctuation(&out)
}

/// Colapsa espaços e remove pontuação de extremidade.
fn trim_punctuation(text: &str) -> String {
    let collapsed = text.split_whitespace().collect::<Vec<_>>().join(" ");
    collapsed
        .trim_matches(|ch: char| ch.is_ascii_punctuation() || ch.is_whitespace())
        .to_owned()
}

/// Corta o texto em `limit` caracteres, na fronteira de palavra mais próxima.
fn truncate_chars(text: &str, limit: usize) -> String {
    if text.chars().count() <= limit {
        return text.to_owned();
    }
    let mut cut = String::new();
    let mut length = 0usize;
    for word in text.split_whitespace() {
        let word_len = word.chars().count();
        let next = if length == 0 {
            word_len
        } else {
            length.saturating_add(1).saturating_add(word_len)
        };
        if next > limit {
            break;
        }
        if length > 0 {
            cut.push(' ');
        }
        cut.push_str(word);
        length = next;
    }
    if cut.is_empty() {
        text.chars().take(limit).collect()
    } else {
        cut
    }
}

#[cfg(test)]
mod tests {
    use super::{MAX_CHARS, MAX_WORDS, title_from, title_of};
    use crate::session::{Event, SessionLog};

    #[test]
    fn keeps_the_first_words_and_drops_noise() {
        let title = title_from("<pedido>Arranja o bug do parser</pedido>").unwrap_or_default();
        assert_eq!(title, "Arranja o bug do parser");
    }

    #[test]
    fn caps_the_number_of_words() {
        let prompt = "um dois tres quatro cinco seis sete oito nove dez";
        let title = title_from(prompt).unwrap_or_default();
        assert_eq!(title.split_whitespace().count(), MAX_WORDS);
        assert_eq!(title, "um dois tres quatro cinco seis sete oito");
    }

    #[test]
    fn caps_the_length_at_a_word_boundary() {
        let prompt = "aaaaaaaaaa bbbbbbbbbb cccccccccc dddddddddd eeeeeeeeee ffffffffff";
        let title = title_from(prompt).unwrap_or_default();
        assert!(title.chars().count() <= MAX_CHARS);
        assert!(!title.ends_with(' '));
    }

    #[test]
    fn ignores_code_fences_and_empty_prompts() {
        assert_eq!(title_from("```rust\nfn main() {}\n```"), None);
        assert_eq!(title_from("   \n\n"), None);
        assert_eq!(title_from("..."), None);
        let mixed = title_from("explica isto\n```\nsegredo\n```\nfim").unwrap_or_default();
        assert_eq!(mixed, "explica isto fim");
    }

    #[test]
    fn title_of_uses_the_first_turn_and_is_stable() {
        let mut log = SessionLog::new();
        assert_eq!(title_of(&log), None);
        log.push(Event::TurnStart {
            prompt: "primeiro pedido".to_owned(),
        });
        log.push(Event::TurnStart {
            prompt: "segundo pedido".to_owned(),
        });
        assert_eq!(title_of(&log).as_deref(), Some("primeiro pedido"));
    }
}
