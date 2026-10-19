//! Acções (subcomandos de 2.º nível) dos verbos do `ktu`.
//!
//! Separadas de [`crate::cli`] (que fica com o mapa de verbos) para manter os
//! ficheiros pequenos e o mapa legível num só lugar.

use std::path::PathBuf;

use clap::Subcommand;

use crate::memory_drain::DrainMode;

/// Acções de gestão de `ktu session` (o REPL é a acção por omissão).
#[derive(Debug, Subcommand)]
pub(crate) enum SessionAction {
    /// Lista as sessões persistidas.
    Ls {
        /// *Root* do projecto (por omissão, o directório actual).
        #[arg(long)]
        root: Option<PathBuf>,
    },
    /// Remove uma sessão (log, checkpoint e histórico).
    Rm {
        /// Identificador da sessão.
        id: String,
        /// *Root* do projecto (por omissão, o directório actual).
        #[arg(long)]
        root: Option<PathBuf>,
    },
    /// Copia uma sessão para um novo id.
    Fork {
        /// Identificador de origem.
        from: String,
        /// Identificador de destino.
        to: String,
        /// *Root* do projecto (por omissão, o directório actual).
        #[arg(long)]
        root: Option<PathBuf>,
    },
}

/// Acções de `ktu agents` (agentes declarativos em `.agents/agents/`).
#[derive(Debug, Subcommand)]
pub(crate) enum AgentsAction {
    /// Lista os agentes declarativos e as suas tools efectivas.
    Ls {
        /// *Root* do projecto (por omissão, o directório actual).
        #[arg(long)]
        root: Option<PathBuf>,
    },
}

/// Acções de `ktu commands` (comandos do projecto em `.agents/commands/`).
#[derive(Debug, Subcommand)]
pub(crate) enum CommandsAction {
    /// Lista os comandos do projecto (invocáveis por `/nome` na sessão).
    Ls {
        /// *Root* do projecto (por omissão, o directório actual).
        #[arg(long)]
        root: Option<PathBuf>,
    },
}

/// Acções de `ktu skills` (skills declarativas em `.agents/skills/`).
#[derive(Debug, Subcommand)]
pub(crate) enum SkillsAction {
    /// Lista as skills efectivas (global + projecto, com a origem).
    Ls {
        /// *Root* do projecto (por omissão, o directório actual).
        #[arg(long)]
        root: Option<PathBuf>,
    },
}

/// Acções de `ktu permission`.
#[derive(Debug, Subcommand)]
pub(crate) enum PermissionAction {
    /// Lista as permissões explícitas.
    Ls {
        /// *Root* do projecto (por omissão, o directório actual).
        #[arg(long)]
        root: Option<PathBuf>,
    },
    /// Define a permissão de uma tool (`always|ask|never`).
    Set {
        /// Nome da tool.
        tool: String,
        /// Permissão (`always|ask|never`).
        permission: String,
        /// *Root* do projecto (por omissão, o directório actual).
        #[arg(long)]
        root: Option<PathBuf>,
    },
    /// Remove a permissão de uma tool (volta ao *default*).
    Unset {
        /// Nome da tool.
        tool: String,
        /// *Root* do projecto (por omissão, o directório actual).
        #[arg(long)]
        root: Option<PathBuf>,
    },
}

/// Acções de `ktu self`.
#[derive(Debug, Subcommand)]
pub(crate) enum SelfAction {
    /// Imprime a versão do binário.
    Version,
    /// Actualiza o binário pelo instalador oficial (release + checksum).
    Upgrade,
    /// Gera um script de *completions* para a shell indicada.
    Completions {
        /// Shell alvo (`bash|zsh|fish|powershell|elvish`).
        shell: String,
    },
    /// Mede a superfície pública (subcomandos, flags e tools) — o teto do `surface.toml`.
    Surface,
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

/// Acções de `ktu config`.
#[derive(Debug, Subcommand)]
pub(crate) enum ConfigAction {
    /// Lista a configuração efectiva (ou um âmbito).
    List,
    /// Lê uma chave.
    Get {
        /// Chave (`provider`, `base_url`, `model`, `effort`, `max_steps`).
        key: String,
    },
    /// Define uma chave.
    Set {
        /// Chave.
        key: String,
        /// Valor.
        value: String,
    },
    /// Remove uma chave.
    Unset {
        /// Chave.
        key: String,
    },
    /// Diagnostica o schema (versão, migração pendente) — **sem escrever**.
    Doctor,
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
    /// Projecta os logs de sessão em notas derivadas (`session-log.jsonl`).
    Project {
        /// *Root* do projecto (por omissão, o directório actual).
        #[arg(long)]
        root: Option<PathBuf>,
        /// Sessão a projectar (por omissão, todas as de `.ktu/sessions`).
        #[arg(long)]
        session: Option<String>,
        /// Destino do JSONL derivado (por omissão, `.agents/memory/session-log.jsonl`).
        #[arg(long)]
        out: Option<PathBuf>,
    },
    /// Diagnostica a memória (âncoras quebradas, saúde do índice) — read-only.
    Doctor {
        /// *Root* do projecto (por omissão, o directório actual).
        #[arg(long)]
        root: Option<PathBuf>,
    },
    /// Drena a fila derivada de embeddings e gere o worker (`service`).
    Drain {
        /// *Root* do projecto (por omissão, o directório actual).
        #[arg(long)]
        root: Option<PathBuf>,
        /// Modo do dreno (`--status` é o *default*).
        #[command(flatten)]
        flags: DrainFlags,
        /// Worker de auto-drain (agendador + servidor de embeddings).
        #[command(subcommand)]
        command: Option<DrainCommand>,
    },
}

/// Flags de `ktu memory drain` (uma só de cada vez; `--status` é o *default*).
#[derive(Debug, clap::Args)]
#[allow(
    clippy::struct_excessive_bools,
    reason = "flags de CLI: a acao e mutuamente exclusiva via clap group"
)]
pub(crate) struct DrainFlags {
    /// Reporta o estado da fila sem drenar (*default*).
    #[arg(long, group = "mode")]
    pub status: bool,
    /// Digere a fila em lotes (grava o índice derivado).
    #[arg(long, group = "mode")]
    pub digest: bool,
    /// Com `--digest`: ignora o índice e reindexa tudo do zero (último recurso).
    #[arg(long, group = "mode", requires = "digest")]
    pub force: bool,
}

impl DrainFlags {
    /// Modo pedido (`--status` é o *default*).
    #[must_use]
    pub(crate) fn mode(&self) -> DrainMode {
        if self.force {
            DrainMode::Force
        } else if self.digest {
            DrainMode::Digest
        } else {
            DrainMode::Status
        }
    }
}

/// Flags de `ktu memory drain service` (uma só acção, via `clap group`).
#[derive(Debug, clap::Args)]
#[allow(
    clippy::struct_excessive_bools,
    reason = "flags de CLI: a acao e mutuamente exclusiva via clap group"
)]
pub(crate) struct ServiceArgs {
    /// Instala o agendador, o servidor de embeddings e cadastra o projecto.
    #[arg(long, group = "action")]
    pub install: bool,
    /// Cadastra o projecto actual (multi-projecto).
    #[arg(long, group = "action")]
    pub subscribe: bool,
    /// Descadastra o projecto actual (mantém o sistema instalado).
    #[arg(long, group = "action")]
    pub unsubscribe: bool,
    /// Mostra a saúde do worker (agendador, servidor, fila). É o *default*.
    #[arg(long, group = "action")]
    pub status: bool,
    /// Remove o sistema (agendador + servidor + binário do worker).
    #[arg(long, group = "action")]
    pub uninstall: bool,
    /// Alinha `memory.toml` ao worker instalado.
    #[arg(long, group = "action")]
    pub reconcile: bool,
    /// Ao desinstalar, preserva o GGUF (*default*).
    #[arg(long, group = "uninstall_model")]
    pub keep_model: bool,
    /// Ao desinstalar, move o GGUF para o *trash* (recuperável; nunca `rm`).
    #[arg(long, group = "uninstall_model")]
    pub remove_model: bool,
    /// Só mostra o plano (não descarrega nem executa).
    #[arg(long)]
    pub dry_run: bool,
    /// Período do dreno (`30m`, `1h`, `1d`).
    #[arg(long, value_name = "DUR", default_value = "1h")]
    pub every: String,
    /// Porta do servidor de embeddings local.
    #[arg(long, value_name = "N", default_value_t = 8889)]
    pub port: u16,
    /// Caminho do GGUF (por omissão, ao lado do `idle.conf` global).
    #[arg(long, value_name = "PATH")]
    pub model: Option<String>,
    /// Não descarrega o modelo no `--install` (falha se faltar).
    #[arg(long)]
    pub no_deps: bool,
}

/// Subcomando de `ktu memory drain`.
#[derive(Debug, Subcommand)]
pub(crate) enum DrainCommand {
    /// Gere o worker de auto-drain (agendador + servidor de embeddings).
    Service(ServiceArgs),
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
