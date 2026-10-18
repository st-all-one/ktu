//! Ids **estáveis** de evento — o contrato de máquina do log.
//!
//! Nunca se loga texto livre como contrato: o `event` é um destes ids e os
//! detalhes vão em campos estruturados (`LogRecord::fields`). Acrescentar um
//! evento é aditivo; renomear um id é uma quebra de contrato.

/// Sessão iniciada.
pub const SESSION_START: &str = "session.start";
/// Sessão retomada a partir de um *checkpoint*.
pub const SESSION_RESUME: &str = "session.resume";
/// Sessão terminada adequadamente.
pub const SESSION_END: &str = "session.end";
/// Turno iniciado.
pub const TURN_START: &str = "turn.start";
/// Turno terminado (com `StopReason`).
pub const TURN_END: &str = "turn.end";
/// Pedido enviado ao modelo.
pub const MODEL_REQUEST: &str = "model.request";
/// Saída do modelo (delta ou fim).
pub const MODEL_OUTPUT: &str = "model.output";
/// Tool chamada a ser executada.
pub const TOOL_CALL: &str = "tool.call";
/// Resultado de tool.
pub const TOOL_RESULT: &str = "tool.result";
/// Tool **diferida** carregada por `tool_search` (passa ao catálogo do próximo pedido).
pub const TOOL_LOADED: &str = "tool.loaded";
/// Instruções locais (`AGENTS.md` mais próximo) anexadas a uma leitura.
pub const CONTEXT_LOCAL: &str = "context.local";
/// Decisão de guardrail (`Allow`/`Deny`/`Escalate`) — a evidência da tese.
pub const GUARD_DECISION: &str = "guard.decision";
/// Erro interno do kernel.
pub const KERNEL_ERROR: &str = "kernel.error";
/// Memória recordada no arranque de sessão (`hints` são ponteiros).
pub const MEMORY_RECALL: &str = "memory.recall";
/// Escrita automática de memória no fim do turno (gatilho `session_end`).
pub const MEMORY_WRITE: &str = "memory.write";
/// Contexto compactado (resumo dos turnos antigos) — `compacted: true`.
pub const SESSION_COMPACT: &str = "session.compact";
/// Servidor da porta iniciado (endereço em `bind`).
pub const SERVER_START: &str = "server.start";
/// *Dispatch* de um hook para os plugins (evidência da intercepção — §2.9).
pub const PLUGIN_DISPATCH: &str = "plugin.dispatch";
/// Ciclo de vida de um plugin (carregado, recarregado, errado — §2.9).
pub const PLUGIN_LIFECYCLE: &str = "plugin.lifecycle";
/// Mensagem de *steering* entregue ao modelo numa fronteira segura.
pub const STEER: &str = "turn.steer";

/// Todos os ids conhecidos, para validação e *goldens*.
pub const ALL: &[&str] = &[
    SESSION_START,
    SESSION_RESUME,
    SESSION_END,
    TURN_START,
    TURN_END,
    MODEL_REQUEST,
    MODEL_OUTPUT,
    TOOL_CALL,
    TOOL_RESULT,
    GUARD_DECISION,
    KERNEL_ERROR,
    MEMORY_RECALL,
    MEMORY_WRITE,
    SESSION_COMPACT,
    SERVER_START,
    PLUGIN_DISPATCH,
    PLUGIN_LIFECYCLE,
    STEER,
];
