//! Definição da linha de comandos (`clap`) do `ktu`.
//!
//! Separada de `main.rs` (que fica só com o despacho) para manter ficheiros
//! pequenos e o mapa de comandos legível num só lugar. Os subcomandos estão
//! **agrupados por área** (a ordem do `Commands:` segue as áreas do rodapé).

use std::path::PathBuf;

use clap::{Parser, Subcommand};

pub(crate) use crate::actions::{
    AgentsAction, CommandsAction, ConfigAction, GuardrailsAction, MemoryAction, PermissionAction,
    PluginsAction, ProvidersAction, SelfAction, SessionAction, TrashAction,
};

/// Rodapé da ajuda: as **áreas** do CLI e o ciclo (estilo `kd`).
const AFTER_HELP: &str = "\
ÁREAS:
  CONVERSA      run · session
  CONHECIMENTO  memory · guardrails · permission · commands · agents · plugins · trash
  PROJECTO      init · doctor · mode · prompt
  SISTEMA       prime · self · providers
  PORTA         serve · tui

CICLO: entender → agir → verificar; memória: ask antes, write depois.
MEMÓRIA: `.agents/memory/notes.jsonl` é a fonte; `.idx/` é derivado. O canal semântico é
  opcional: `ktu memory drain --status|--digest|--force` (worker: `... drain service --install`).
SAÍDA: stdout = dados · stderr = logs · --json = envelope {success,command,data?,error?}.
EXIT: 0 = ok · 1 = falha · 2 = uso inválido.
veja: ktu help <verbo>";

/// `ktu` — agente mínimo: sessão, contexto e cache atrás de uma porta única.
#[derive(Debug, Parser)]
#[command(name = "ktu", version, about, long_about = None, after_help = AFTER_HELP)]
#[allow(
    clippy::struct_excessive_bools,
    reason = "flags booleanas da linha de comandos: --json e --quiet"
)]
pub(crate) struct Cli {
    /// Emite a saída no envelope JSON (contrato de máquina).
    #[arg(long, global = true)]
    pub(crate) json: bool,
    /// Silencia os logs em `stderr`.
    #[arg(long, global = true)]
    pub(crate) quiet: bool,
    /// Nível de log em `stderr` (`error|warn|info|debug|trace|off`).
    #[arg(long, global = true, env = "KTU_LOG_LEVEL", default_value = "info")]
    pub(crate) log_level: String,
    /// Subcomando a executar (por omissão, mostra a ajuda).
    #[command(subcommand)]
    pub(crate) command: Option<Command>,
}

/// Subcomandos do `ktu`, agrupados por **área** (a ordem é a do `--help`).
#[derive(Debug, Subcommand)]
#[allow(
    clippy::large_enum_variant,
    reason = "variante Run agrega os argumentos da linha de comandos; trivial e unica"
)]
pub(crate) enum Command {
    // ── CONVERSA ────────────────────────────────────────────────────────────
    /// Corre um turno do agente contra o `llama-server` (ou `--provider mock`).
    Run {
        /// Instrução do utilizador (palavras; `-` ou vazio com *pipe* lê de `stdin`).
        ///
        /// Com `--command`, estas palavras são os **argumentos** do comando.
        prompt: Vec<String>,
        /// Comando do projecto (`.agents/commands/*.md`) a correr; o prompt é o
        /// corpo do comando, com `$1`…`$9`/`$ARGUMENTS` substituídos.
        #[arg(long, conflicts_with = "params")]
        command: Option<String>,
        /// Objeto JSON com todos os parâmetros do turno (`-` lê de `stdin`).
        #[arg(long)]
        params: Option<String>,
        /// Lote JSONL de `--params`, um por linha (`-` lê de `stdin`).
        #[arg(long)]
        batch: Option<String>,
        /// Com `--batch`: corre tudo na mesma sessão, em ordem (default: paralelo).
        #[arg(long)]
        sequential: bool,
        /// Com `--batch` paralelo: concorrência máxima.
        #[arg(long, default_value_t = 4)]
        concurrency: usize,
        /// *Root* da sessão (por omissão, o directório actual).
        #[arg(long)]
        root: Option<PathBuf>,
        /// Provider: `llama`, `opencode-go` (remoto), `mock` (eco) ou `scripted` (guião).
        #[arg(long, env = "KTU_PROVIDER")]
        provider: Option<String>,
        /// Guião do `--provider scripted` (JSONL: uma `Completion` por linha);
        /// avança a cada passo do modelo, incluindo entre turnos da mesma sessão.
        #[arg(long)]
        script: Option<PathBuf>,
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
        #[arg(long, env = "KTU_EFFORT")]
        effort: Option<String>,
        /// Tecto de passos do turno.
        #[arg(long, env = "KTU_MAX_STEPS")]
        max_steps: Option<u32>,
        /// Identificador da sessão persistida (`.ktu/sessions/<id>.jsonl`); sem ele o
        /// turno é **efémero** (1 turno, sem persistir).
        #[arg(long, env = "KTU_SESSION")]
        session: Option<String>,
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
    /// Abre um REPL multi-turno (front primário) ou gere as sessões persistidas.
    #[command(args_conflicts_with_subcommands = true)]
    Session {
        /// Identificador da sessão a abrir (por omissão, `default`).
        id: Option<String>,
        /// *Root* do projecto (por omissão, o directório actual).
        #[arg(long)]
        root: Option<PathBuf>,
        /// Provider: `llama`, `opencode-go` (remoto) ou `mock` (eco, sem modelo).
        #[arg(long, env = "KTU_PROVIDER")]
        provider: Option<String>,
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
        #[arg(long, env = "KTU_EFFORT")]
        effort: Option<String>,
        /// Tecto de passos por turno.
        #[arg(long, env = "KTU_MAX_STEPS")]
        max_steps: Option<u32>,
        /// Arranca em modo plano (read-only + tool `plan`) — §2.13.
        #[arg(long)]
        plan: bool,
        /// Acção de gestão (por omissão, abre o REPL).
        #[command(subcommand)]
        action: Option<SessionAction>,
    },
    // ── CONHECIMENTO ────────────────────────────────────────────────────────
    /// Inspecciona e mantém a memória persistente do projecto.
    Memory {
        /// Acção sobre a memória.
        #[command(subcommand)]
        action: MemoryAction,
    },
    /// Inspecciona os guardrails declarativos (§2.8).
    Guardrails {
        /// Acção sobre os guardrails.
        #[command(subcommand)]
        action: GuardrailsAction,
    },
    /// Inspecciona os comandos do projecto (`.agents/commands/*.md`).
    Commands {
        /// Acção sobre os comandos.
        #[command(subcommand)]
        action: CommandsAction,
    },
    /// Inspecciona os agentes declarativos (`.agents/agents/*.md`).
    Agents {
        /// Acção sobre os agentes.
        #[command(subcommand)]
        action: AgentsAction,
    },
    /// Inspecciona e edita as permissões por tool (`Always`/`Ask`/`Never`).
    Permission {
        /// Acção sobre as permissões.
        #[command(subcommand)]
        action: PermissionAction,
    },
    /// Inspecciona e edita a configuração (global + local).
    Config {
        /// Opera sobre a config local (`.agents/config.toml`).
        #[arg(long, global = true, conflicts_with = "global")]
        local: bool,
        /// Opera sobre a config global (`~/.config/ktu/config.toml`).
        #[arg(long, global = true, conflicts_with = "local")]
        global: bool,
        /// Acção sobre a configuração.
        #[command(subcommand)]
        action: ConfigAction,
    },
    /// Inspecciona e recarrega os plugins externos (§2.9).
    Plugins {
        /// Acção sobre os plugins.
        #[command(subcommand)]
        action: PluginsAction,
    },
    /// Gere o *trash* (ficheiros movidos em vez de apagados).
    Trash {
        /// Acção sobre o *trash*.
        #[command(subcommand)]
        action: TrashAction,
    },
    // ── PROJECTO ────────────────────────────────────────────────────────────
    /// Prepara o ambiente do projecto (layout, skill, git, `AGENTS.md`) — idempotente.
    Init {
        /// Reescreve os ficheiros gerados (skill, config); nunca a memória.
        #[arg(long)]
        force: bool,
        /// Exclui `.agents/` e `.ktu/` do git (local-only).
        #[arg(long, conflicts_with = "git_tracked")]
        git_excluded: bool,
        /// Versiona `.agents/`; exclui só o derivado e o runtime (por omissão).
        #[arg(long, conflicts_with = "git_excluded")]
        git_tracked: bool,
        /// Não imprime o protocolo no fim.
        #[arg(long)]
        no_prompt: bool,
    },
    /// Mostra o system prompt efectivo (directriz + `AGENTS.md` + acrescento).
    Prompt {
        /// *Root* do projecto (por omissão, o directório actual).
        #[arg(long)]
        root: Option<PathBuf>,
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
    // ── SISTEMA ─────────────────────────────────────────────────────────────
    /// Imprime o protocolo do agente (byte-idêntico por versão).
    Prime,
    /// Utilitários do binário (versão, *upgrade*, superfície).
    #[command(name = "self")]
    SelfCmd {
        /// Acção sobre o binário.
        #[command(subcommand)]
        action: SelfAction,
    },
    /// Diagnostica o provider `llama.cpp` (`llama-server`).
    Providers {
        /// Acção de diagnóstico.
        #[command(subcommand)]
        action: ProvidersAction,
    },
    // ── PORTA ───────────────────────────────────────────────────────────────
    /// Serve a porta única (HTTP/1.1 + HTTP/2 + SSE) para front-ends externos.
    Serve {
        /// *Root* do projecto (por omissão, o directório actual).
        #[arg(long)]
        root: Option<PathBuf>,
        /// Endereço de escuta (`host:porta`).
        #[arg(long, env = "KTU_BIND", default_value = "127.0.0.1:7717")]
        bind: String,
        /// Provider: `llama`, `opencode-go` (remoto) ou `mock` (eco, sem modelo).
        #[arg(long, env = "KTU_PROVIDER")]
        provider: Option<String>,
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
        #[arg(long, env = "KTU_EFFORT")]
        effort: Option<String>,
        /// Tecto de passos por turno.
        #[arg(long, env = "KTU_MAX_STEPS")]
        max_steps: Option<u32>,
        /// Arranca as sessões em modo plano (read-only + tool `plan`) — §2.13.
        #[arg(long)]
        plan: bool,
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
}
