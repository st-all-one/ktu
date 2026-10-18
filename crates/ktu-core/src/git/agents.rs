//! `AGENTS.md` gerido: bloco de protocolo com marcadores idempotentes.
//!
//! O bloco é um *router* conciso (aponta para `ktu prime`); o protocolo completo
//! nunca é duplicado no ficheiro do projecto.

use super::block;

/// Início do bloco gerido.
pub const MARKER_BEGIN: &str = "<!-- ktu:start -->";
/// Fim do bloco gerido.
pub const MARKER_END: &str = "<!-- ktu:end -->";
/// Prefixo do marcador de versão.
pub const VERSION_PREFIX: &str = "<!-- ktu:version:";
/// Versão do protocolo embutido no `AGENTS.md`.
pub const VERSION: u32 = 1;
/// Nome do ficheiro.
pub const FILE: &str = "AGENTS.md";

/// Bloco de protocolo que o `ktu init` escreve no `AGENTS.md`.
#[must_use]
pub fn protocol_block() -> String {
    format!(
        "{MARKER_BEGIN}\n\
         {VERSION_PREFIX} {VERSION} -->\n\
         # ktu — agente mínimo\n\
         \n\
         `.agents/` é conhecimento (versionável: skills, guardrails, memória); `.ktu/` é\n\
         runtime (sessões, plugins, trash, planos). Este ficheiro é o *router*.\n\
         \n\
         Ciclo: entender → planear → editar → correr → verificar. Consulta a memória antes\n\
         de decidir (`memory_ask`); grava cada aprendizado (`memory_write`).\n\
         \n\
         Protocolo completo: `ktu prime`. Diagnóstico: `ktu doctor`.\n\
         {MARKER_END}\n"
    )
}

/// Extrai a versão do protocolo presente no texto, se houver.
#[must_use]
pub fn version_in(text: &str) -> Option<u32> {
    for line in text.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix(VERSION_PREFIX) {
            let digits: String = rest
                .trim_start()
                .chars()
                .take_while(char::is_ascii_digit)
                .collect();
            if let Ok(value) = digits.parse::<u32>() {
                return Some(value);
            }
        }
    }
    None
}

/// Garante (ou actualiza) o bloco de protocolo no texto do `AGENTS.md`.
#[must_use]
pub fn upsert(original: &str) -> String {
    block::upsert(original, MARKER_BEGIN, MARKER_END, Some(&protocol_block()))
}

#[cfg(test)]
mod tests {
    use super::{MARKER_BEGIN, MARKER_END, VERSION, protocol_block, upsert, version_in};

    #[test]
    fn block_declares_current_version_and_markers() {
        let block = protocol_block();
        assert_eq!(version_in(&block), Some(VERSION));
        assert!(block.contains(MARKER_BEGIN));
        assert!(block.contains(MARKER_END));
    }

    #[test]
    fn upsert_is_idempotent_and_preserves_the_rest() {
        let original = "# AGENTS\n\nRegras do projecto.\n";
        let once = upsert(original);
        assert!(once.contains("Regras do projecto."));
        assert!(once.contains(MARKER_BEGIN));
        assert_eq!(once, upsert(&once));
    }

    #[test]
    fn version_absent_is_none() {
        assert_eq!(version_in("# AGENTS\n"), None);
    }
}
