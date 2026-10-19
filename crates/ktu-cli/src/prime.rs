//! `ktu prime` — o protocolo do agente (byte-idêntico por versão).
//!
//! É o texto que ensina o modelo a usar o ktu: tools, ciclo, memória, guardrails
//! e contrato de saída. Estável por versão — nunca muda sem um *bump* consciente
//! (o protocolo é a interface com o modelo, não um texto livre).

use std::process::ExitCode;

use crate::output::{self, Format};

/// Protocolo do agente — **byte-idêntico por versão** (constante compilada).
pub(crate) const PROTOCOL: &str = "\
ktu — agente mínimo (kernel puro; efeitos por portas).

CICLO: entender → planear → editar → correr → verificar.
  Consulta a memória (`memory_ask`) antes de decidir; grava cada aprendizado
  (`memory_write`) — uma afirmação por nota.

FERRAMENTAS:
  read_file    Lê um ficheiro (com continuação quando truncado).
  write_file   Cria ou sobrescreve um ficheiro.
  edit_file    Edita por blocos (atómico: tudo ou nada).
  list_dir     Lista um directório.
  glob         Encontra ficheiros por padrão.
  grep         Procura em ficheiros.
  bash         Executa um comando (timeout + cap).
  plan         Mantém o plano de tarefas (modo plano).
  memory_ask   Consulta a memória do projecto (devolve ponteiros).
  memory_write Grava conhecimento durável.
  trash_file   Move para o trash (recuperável) em vez de apagar.
  subagent     Delega exploração a um subagente (nomes em AGENTES).
  tool_search  Procura tools fora do catálogo e carrega-as para o próximo passo
               (só existe quando o projecto difere tools em `.agents/config.toml`).

MEMÓRIA: `.agents/memory/notes.jsonl` é a fonte da verdade; o índice é derivado.
  Ancora o código relevante (`anchors`); usa `kind` e `tags` quando aplicável.
  `ktu memory project` projeta `.ktu/sessions/<id>.jsonl` em notas derivadas
  (`.agents/memory/session-log.jsonl`, `activity=derive`) — consultáveis por `kd ai session-log`.
  Canal semântico (opcional, off por omissão): `ktu memory drain --status|--digest|--force`
  (worker: `ktu memory drain service --install`); sem ele o recall é lexical.

MODO: `plan` é read-only estrutural; `build` altera. A transição plan→build exige
  aprovação do utilizador — o modelo nunca muda de modo sozinho.

SESSÃO: `ktu run` é **efémero** por omissão (1 turno, sem persistir); `--session <id>`
  persiste em `.ktu/sessions/<id>.jsonl` (com lock por sessão); `--resume` implica `default`.
  O título da sessão é derivado do primeiro pedido (≤ 8 palavras) e aparece em `ktu session ls`.

GUARDRAILS: `.agents/guardrails/*.md` (fail-closed). Toda decisão é evidência no log
  (`guard.decision`). Uma tool negada devolve erro tipado + dica; nunca insiste.

CONTEXTO: um só dono do tecto (BudgetGate); a instrução do utilizador nunca cai.

CONVENÇÕES: `.agents/` = conhecimento (versionável) · `.ktu/` = runtime · `AGENTS.md` =
  instruções do projecto (escritas por ti; o ktu só lê).
  Instruções por directório: `AGENTS.md` na raiz e aninhados (entram no prompt, raiz primeiro;
  o mais próximo vem depois). Directriz: `.agents/SYSTEM.md` substitui a base do agente e
  `.agents/APPEND_SYSTEM.md` acrescenta-lhe (`<global>/SYSTEM.md` é o nível do utilizador; o
  projecto ganha). Comandos do projecto: `.agents/commands/*.md`, invocáveis por `/nome args`
  (`$ARGUMENTS` no corpo); `ktu commands ls` lista-os. `ktu prompt` mostra o prompt efectivo.
AGENTES: o registo traz `ktu` (primário), `explorer` (read-only) e os agentes declarativos de
  `.agents/agents/*.md` (`mode: primary|subagent`, `tools` allow/deny). A tool `subagent`
  enumera os nomes disponíveis; `ktu agents ls` mostra as tools efectivas.
INSTRUÇÕES LOCAIS: ao leres um ficheiro, as regras do `AGENTS.md` mais próximo (desse directório
  ou de um pai) são anexadas ao resultado — aplica-as só àquele âmbito.
TOOLS DIFERIDAS: `deferred_tools` em `.agents/config.toml` tira tools do catálogo; usa `tool_search`
  para as descobrir (ficam disponíveis no passo seguinte).
RECEITAS: `ktu run --command <nome> [args]` corre um comando de `.agents/commands/*.md` (com as
  suas `settings`); linhas `!cmd` só correm com `shell: true` e depois de passarem pelos guardrails.

SAÍDA: stdout = dados; stderr = logs; ids de evento estáveis.
EXIT: 0 = ok · 1 = falha · 2 = uso inválido.
";

/// Imprime o protocolo em `stdout` (dado, não log) — texto ou envelope JSON.
pub(crate) fn run(format: Format) -> ExitCode {
    output::emit(
        format,
        "prime",
        PROTOCOL.trim_end(),
        &serde_json::json!({ "protocol": PROTOCOL }),
    );
    ExitCode::SUCCESS
}

#[cfg(test)]
mod tests {
    use super::PROTOCOL;

    #[test]
    fn protocol_is_non_empty_and_ends_with_newline() {
        assert_ne!(PROTOCOL, "");
        assert!(PROTOCOL.ends_with('\n'));
    }

    #[test]
    fn protocol_names_every_tool() {
        for tool in [
            "read_file",
            "write_file",
            "edit_file",
            "list_dir",
            "glob",
            "grep",
            "bash",
            "plan",
            "memory_ask",
            "memory_write",
            "trash_file",
            "subagent",
        ] {
            assert!(PROTOCOL.contains(tool), "falta a tool {tool}");
        }
    }

    #[test]
    fn protocol_declares_the_output_contract() {
        assert!(PROTOCOL.contains("stdout = dados"));
        assert!(PROTOCOL.contains("stderr = logs"));
    }

    #[test]
    fn protocol_documents_the_ephemeral_default() {
        assert!(PROTOCOL.contains("efémero"));
        assert!(PROTOCOL.contains("--session"));
    }

    #[test]
    fn protocol_documents_the_project_conventions() {
        for needle in [
            "AGENTS.md",
            "SYSTEM.md",
            "APPEND_SYSTEM.md",
            ".agents/commands/",
            "$ARGUMENTS",
            "ktu commands ls",
            ".agents/agents/",
            "ktu agents ls",
            "tool_search",
            "deferred_tools",
            "INSTRUÇÕES LOCAIS",
            "--command",
        ] {
            assert!(PROTOCOL.contains(needle), "falta {needle}");
        }
    }
}
