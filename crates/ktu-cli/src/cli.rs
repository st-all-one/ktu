//! Definição da linha de comandos (`clap`) do `ktu`.
//!
//! Separada de `main.rs` (que fica só com o despacho) para manter ficheiros
//! pequenos e o mapa de comandos legível num só lugar.

use std::path::PathBuf;

use clap::{Parser, Subcommand};

/// `ktu` — agente mínimo: sessão, contexto e cache atrás de uma porta única.
#[derive(Debug, Parser)]
#[command(name = "ktu", version, about, long_about = None)]
pub(crate) struct Cli {
    /// Subcomando a executar (por omissão, mostra a ajuda).
    #[command(subcommand)]
    pub(crate) command: Option<Command>,
}

/// Subcomandos do `ktu`.
#[derive(Debug, Subcommand)]
#[allow(
    clippy::large_enum_variant,
    reason = "variante Run agrega os argumentos da linha de comandos; trivial e unica"
)]
pub(crate) enum Command {
    /// Inicializa o layout `.agents/` (conhecimento) e `.ktu/` (runtime).
    Init,
    /// Corre um turno do agente contra o `llama-server` (ou `--provider mock`).
    Run {
        /// Instrução do utilizador (palavras).
        prompt: Vec<String>,
        /// *Root* da sessão (por omissão, o directório actual).
        #[arg(long)]
        root: Option<PathBuf>,
        /// Provider: `llama`, `opencode-go` (remoto) ou `mock` (eco, sem modelo).
        #[arg(long, env = "KTU_PROVIDER", default_value = "llama")]
        provider: String,
        /// *Base URL* do provider (por omissão, derivada do provider).
        #[arg(long, env = "KTU_BASE_URL")]
        base_url: Option<String>,
        /// Nome/alias do modelo (por omissão, derivado do provider).
        #[arg(long, env = "KTU_MODEL")]
        model: Option<String>,
        /// Chave do gateway remoto (`opencode-go`); por omissão do ambiente.
        #[arg(long, env = "OPENCODE_GO_API_KEY", hide_env_values = true)]
        api_key: Option<String>,
        /// Esforço de raciocínio (`low|medium|high|xhigh`).
        #[arg(long, env = "KTU_EFFORT", default_value = "xhigh")]
        effort: String,
        /// Tecto de passos do turno.
        #[arg(long, env = "KTU_MAX_STEPS", default_value_t = 24)]
        max_steps: u32,
        /// Identificador da sessão persistida (`.ktu/sessions/<id>.jsonl`).
        #[arg(long, env = "KTU_SESSION", default_value = "default")]
        session: String,
        /// Retoma a sessão (carrega o log) antes deste turno.
        #[arg(long)]
        resume: bool,
        /// Força a compactação do contexto antes do turno (implica retomar).
        #[arg(long)]
        compact: bool,
        /// Arranca em modo plano (read-only + tool `plan`) — §2.13.
        #[arg(long)]
        plan: bool,
    },
    /// Serve a porta única (HTTP/1.1 + HTTP/2 + SSE) para front-ends externos.
    Serve {
        /// *Root* do projecto (por omissão, o directório actual).
        #[arg(long)]
        root: Option<PathBuf>,
        /// Endereço de escuta (`host:porta`).
        #[arg(long, env = "KTU_BIND", default_value = "127.0.0.1:7717")]
        bind: String,
        /// Provider: `llama`, `opencode-go` (remoto) ou `mock` (eco, sem modelo).
        #[arg(long, env = "KTU_PROVIDER", default_value = "llama")]
        provider: String,
        /// *Base URL* do provider (por omissão, derivada do provider).
        #[arg(long, env = "KTU_BASE_URL")]
        base_url: Option<String>,
        /// Nome/alias do modelo (por omissão, derivado do provider).
        #[arg(long, env = "KTU_MODEL")]
        model: Option<String>,
        /// Chave do gateway remoto (`opencode-go`); por omissão do ambiente.
        #[arg(long, env = "OPENCODE_GO_API_KEY", hide_env_values = true)]
        api_key: Option<String>,
        /// Esforço de raciocínio (`low|medium|high|xhigh`).
        #[arg(long, env = "KTU_EFFORT", default_value = "xhigh")]
        effort: String,
        /// Tecto de passos por turno.
        #[arg(long, env = "KTU_MAX_STEPS", default_value_t = 24)]
        max_steps: u32,
        /// Arranca as sessões em modo plano (read-only + tool `plan`) — §2.13.
        #[arg(long)]
        plan: bool,
    },
    /// Diagnostica o provider `llama.cpp` (`llama-server`).
    Providers {
        /// Acção de diagnóstico.
        #[command(subcommand)]
        action: ProvidersAction,
    },
    /// Gere o *trash* (ficheiros movidos em vez de apagados).
    Trash {
        /// Acção sobre o *trash*.
        #[command(subcommand)]
        action: TrashAction,
    },
    /// Inspecciona e mantém a memória persistente do projecto.
    Memory {
        /// Acção sobre a memória.
        #[command(subcommand)]
        action: MemoryAction,
    },
    /// Diagnostica o layout e o conhecimento do projecto.
    Doctor {
        /// *Root* do projecto (por omissão, o directório actual).
        #[arg(long)]
        root: Option<PathBuf>,
        /// Diagnostica a memória (âncoras quebradas, saúde do índice).
        #[arg(long)]
        memory: bool,
    },
    /// Muda o modo da sessão (`SetMode`) — sempre logado e sobrevive a `resume` (I15).
    Mode {
        /// Modo alvo: `plan` ou `build`.
        target: String,
        /// Aprova a transição `plan->build` (obrigatória — I15).
        #[arg(long)]
        approve: bool,
        /// *Root* do projecto (por omissão, o directório actual).
        #[arg(long)]
        root: Option<PathBuf>,
        /// Identificador da sessão.
        #[arg(long, default_value = "default")]
        session: String,
    },
    /// Abre uma TUI mínima ligada a um `ktu serve` (cliente da porta).
    Tui {
        /// URL base do servidor (`ktu serve`).
        #[arg(long, env = "KTU_SERVER", default_value = "http://127.0.0.1:7717")]
        server: String,
        /// Identificador da sessão.
        #[arg(long, default_value = "tui")]
        session: String,
    },
    /// Inspecciona e recarrega os plugins externos (§2.9).
    Plugins {
        /// Acção sobre os plugins.
        #[command(subcommand)]
        action: PluginsAction,
    },
    /// Inspecciona os guardrails declarativos (§2.8).
    Guardrails {
        /// Acção sobre os guardrails.
        #[command(subcommand)]
        action: GuardrailsAction,
    },
}

/// Acções de `ktu plugins`.
#[derive(Debug, Subcommand)]
pub(crate) enum PluginsAction {
    /// Lista os plugins activos e a geração corrente.
    List {
        /// *Root* do projecto (por omissão, o directório actual).
        #[arg(long)]
        root: Option<PathBuf>,
    },
    /// Recarrega os plugins, incrementando a geração (I12).
    Reload {
        /// *Root* do projecto (por omissão, o directório actual).
        #[arg(long)]
        root: Option<PathBuf>,
    },
    /// Diagnostica os plugins (manifestos inválidos).
    Doctor {
        /// *Root* do projecto (por omissão, o directório actual).
        #[arg(long)]
        root: Option<PathBuf>,
    },
}

/// Acções de `ktu guardrails`.
#[derive(Debug, Subcommand)]
pub(crate) enum GuardrailsAction {
    /// Lista as regras declarativas carregadas (fail-closed).
    List {
        /// *Root* do projecto (por omissão, o directório actual).
        #[arg(long)]
        root: Option<PathBuf>,
    },
}

/// Acções de `ktu memory`.
#[derive(Debug, Subcommand)]
pub(crate) enum MemoryAction {
    /// Imprime a vista humana (Markdown) derivada de `notes.jsonl`.
    Export {
        /// *Root* do projecto (por omissão, o directório actual).
        #[arg(long)]
        root: Option<PathBuf>,
    },
    /// Lista uma nota por linha (`id`, tipo e afirmação).
    List {
        /// *Root* do projecto (por omissão, o directório actual).
        #[arg(long)]
        root: Option<PathBuf>,
    },
}

/// Acções de `ktu providers`.
#[derive(Debug, Subcommand)]
pub(crate) enum ProvidersAction {
    /// Verifica ligação, modelo e (opcionalmente) o ficheiro GGUF.
    Doctor {
        /// *Base URL* do `llama-server`.
        #[arg(long, env = "KTU_BASE_URL", default_value = "http://127.0.0.1:8080")]
        base_url: String,
        /// Caminho do GGUF a inspeccionar (magic + metadados).
        #[arg(long)]
        gguf: Option<PathBuf>,
        /// Corre uma sonda funcional com tool-calling.
        #[arg(long)]
        probe: bool,
    },
}

/// Acções de `ktu trash`.
#[derive(Debug, Subcommand)]
pub(crate) enum TrashAction {
    /// Lista as entradas do *trash*.
    List {
        /// *Root* da sessão (por omissão, o directório actual).
        #[arg(long)]
        root: Option<PathBuf>,
    },
    /// Restaura uma entrada para o caminho original.
    Restore {
        /// Identificador da entrada.
        id: String,
        /// *Root* da sessão (por omissão, o directório actual).
        #[arg(long)]
        root: Option<PathBuf>,
    },
}
