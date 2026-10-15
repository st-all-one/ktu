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
];
