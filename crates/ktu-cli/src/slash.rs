//! Comandos `/` do REPL: interpretação da linha e ajuda.
//!
//! Os comandos **embutidos** mapeiam em comandos da porta (`SetMode`, `Cancel`, …);
//! um nome desconhecido é um **comando do projecto** (`.agents/commands/*.md`),
//! resolvido pela borda. Separado de `repl.rs` para manter os ficheiros pequenos.

use std::fmt::Write as _;

use ktu_core::command::Command;

/// Comando `/` do REPL (cada um mapeia num comando da porta, ou num comando do projecto).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Slash {
    /// Ajuda do REPL.
    Help,
    /// Sai do REPL.
    Quit,
    /// Muda para modo plano (`SetMode`).
    Plan,
    /// Volta a `build` com aprovação (`SetMode`).
    Build,
    /// Mostra o modo corrente (`State`).
    Mode,
    /// Cancela o turno em curso (`Cancel`).
    Cancel,
    /// Comando do projecto (`.agents/commands/*.md`), resolvido por nome.
    Custom {
        /// Nome do comando.
        name: String,
        /// Argumentos da linha.
        args: String,
    },
}

/// Ajuda dos comandos **embutidos** do REPL (`/help`).
const HELP: &str = "\
comandos:
  /help            esta ajuda
  /plan            muda para modo plano (SetMode)
  /build           volta a build (com aprovacao)
  /mode            mostra o modo corrente
  /cancel          cancela o turno em curso
  /quit            sai (Ctrl-D tambem)
multi-linha: termina a linha com `\\` para continuar.
durante um turno: escrever envia *steering*; /cancel cancela.";

/// Ajuda completa: embutidos + comandos do projecto.
pub(crate) fn help(commands: &[Command]) -> String {
    let mut text = HELP.to_owned();
    if !commands.is_empty() {
        text.push_str("\ncomandos do projeto (.agents/commands/):\n");
        for command in commands {
            let _ignored = writeln!(text, "  /{:<14} {}", command.name(), command.description());
        }
    }
    text
}

/// Interpreta uma linha como comando `/` (`None` se não começar por `/`).
#[must_use]
pub(crate) fn parse_slash(line: &str) -> Option<Slash> {
    let rest = line.trim().strip_prefix('/')?;
    let (head, tail) = rest.split_once(char::is_whitespace).unwrap_or((rest, ""));
    Some(match head {
        "help" => Slash::Help,
        "quit" | "exit" => Slash::Quit,
        "plan" => Slash::Plan,
        "build" => Slash::Build,
        "mode" => Slash::Mode,
        "cancel" => Slash::Cancel,
        other => Slash::Custom {
            name: other.to_owned(),
            args: tail.trim().to_owned(),
        },
    })
}

#[cfg(test)]
mod tests {
    use super::{Slash, parse_slash};

    #[test]
    fn parses_known_slash_commands() {
        assert_eq!(parse_slash("/quit"), Some(Slash::Quit));
        assert_eq!(parse_slash("  /plan  "), Some(Slash::Plan));
        assert_eq!(parse_slash("/exit"), Some(Slash::Quit));
        assert_eq!(parse_slash("/build extra"), Some(Slash::Build));
    }

    #[test]
    fn unknown_names_become_project_commands_with_arguments() {
        assert_eq!(parse_slash("ola"), None);
        assert_eq!(
            parse_slash("/revisao src/main.rs"),
            Some(Slash::Custom {
                name: "revisao".to_owned(),
                args: "src/main.rs".to_owned(),
            })
        );
        assert_eq!(
            parse_slash("/foo"),
            Some(Slash::Custom {
                name: "foo".to_owned(),
                args: String::new(),
            })
        );
    }
}
