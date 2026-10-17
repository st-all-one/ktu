//! Tipos do domínio do agente: puros, serializáveis, sem I/O.
//!
//! O **log** da sessão (ver [`crate::session`]) é a fonte da verdade; estes tipos
//! são a projecção que o modelo vê e o que as tools produzem.

use serde::{Deserialize, Serialize};

/// Identificador de uma chamada de tool (atribuído pelo modelo).
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ToolCallId(String);

impl ToolCallId {
    /// Cria um identificador a partir de um texto.
    #[must_use]
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Devolve o texto do identificador.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Uma chamada de tool emitida pelo modelo.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolCall {
    /// Identificador (usado para casar com o `ToolResult`).
    pub id: ToolCallId,
    /// Nome da tool.
    pub name: String,
    /// Argumentos em JSON, tal como o modelo os emitiu.
    pub arguments: String,
}

/// Papel de uma mensagem no pedido ao modelo.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    /// Instruções de sistema.
    System,
    /// Entrada do utilizador.
    User,
    /// Saída do modelo.
    Assistant,
    /// Resultado de tool.
    Tool,
}

/// Mensagem na projecção de *wire* (o que é enviado ao modelo).
///
/// Um passo do modelo é **uma** mensagem `Assistant` com N `tool_calls`,
/// seguida de N mensagens `Tool` (nunca um `Assistant` por call — §2.4).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Message {
    /// Instruções de sistema.
    System {
        /// Conteúdo.
        content: String,
    },
    /// Entrada do utilizador.
    User {
        /// Conteúdo.
        content: String,
    },
    /// Saída do modelo (texto e/ou chamadas de tool).
    Assistant {
        /// Texto emitido.
        content: String,
        /// Pensamento do modelo (*thinking*). Reenviado a gateways `OpenAI` que o
        /// exigem (`reasoning_content`); descartado no dialeto do `llama-server`.
        reasoning: String,
        /// Chamadas de tool emitidas no mesmo passo.
        tool_calls: Vec<ToolCall>,
    },
    /// Resultado de uma tool.
    Tool {
        /// Chamada a que este resultado responde.
        tool_call_id: ToolCallId,
        /// Conteúdo (já dentro do envelope *taint* no caminho real).
        content: String,
        /// Se o resultado é um erro (a tool falhou, não o turno).
        is_error: bool,
    },
}

/// Resultado de uma chamada ao modelo: texto, pensamento e/ou chamadas de tool.
///
/// O **pensamento** (`reasoning`) é um campo próprio, nunca misturado com a
/// resposta final. A projecção de *wire* reenvia-o apenas quando o gateway o
/// exige (thinking mode de gateways `OpenAI`); o `llama-server` não o recebe (§2.5).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Completion {
    /// Texto emitido (resposta final, já sem o pensamento).
    pub text: String,
    /// Pensamento do modelo (*thinking*), quando o template o separa.
    pub reasoning: String,
    /// Chamadas de tool emitidas.
    pub tool_calls: Vec<ToolCall>,
}

/// Especificação de uma tool oferecida ao modelo.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ToolSpec {
    /// Nome da tool.
    pub name: String,
    /// Descrição (para o modelo).
    pub description: String,
    /// Parâmetros em JSON Schema, como texto.
    pub parameters: String,
}

/// Razão de término de um turno — **nunca em silêncio** (I5).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StopReason {
    /// O modelo terminou com texto.
    EndTurn,
    /// Atingiu o tecto de passos.
    MaxSteps,
    /// Sem progresso repetido (detecção de estagnação).
    Stalled,
    /// Cancelado pelo utilizador.
    Cancelled,
    /// Erro do kernel ou do provider.
    Error,
}
