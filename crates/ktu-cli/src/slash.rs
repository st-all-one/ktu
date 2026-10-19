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
    /// Força a compactação do contexto (`Cmd::Compact`).
    Compact,
    /// Copia a sessão para um novo id (`/fork <novo>`).
    Fork {
        /// Identificador de destino.
        to: String,
    },
    /// Mostra a memória do projecto (`ktu memory list`).
    Memory,
    /// Retoma outra sessão (`/resume [id]`) — sem id, lista as disponíveis.
    Resume {
        /// Identificador a abrir.
        id: Option<String>,
    },
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
  /compact         forca a compactacao do contexto
  /fork <novo>     copia esta sessao para um novo id
  /memory          mostra a memoria do projecto
  /resume [id]     abre outra sessao (sem id, lista-as)
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
        "compact" => Slash::Compact,
        "memory" => Slash::Memory,
        "fork" => Slash::Fork {
            to: tail.trim().to_owned(),
        },
        "resume" => Slash::Resume {
            id: (!tail.trim().is_empty()).then(|| tail.trim().to_owned()),
        },
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
    fn parses_the_interactive_commands_of_the_phase_3() {
        assert_eq!(parse_slash("/compact"), Some(Slash::Compact));
        assert_eq!(parse_slash("/memory"), Some(Slash::Memory));
        assert_eq!(
            parse_slash("/fork copia"),
            Some(Slash::Fork {
                to: "copia".to_owned()
            })
        );
        assert_eq!(
            parse_slash("/fork"),
            Some(Slash::Fork { to: String::new() })
        );
        assert_eq!(
            parse_slash("/resume outra"),
            Some(Slash::Resume {
                id: Some("outra".to_owned())
            })
        );
        assert_eq!(parse_slash("/resume"), Some(Slash::Resume { id: None }));
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
