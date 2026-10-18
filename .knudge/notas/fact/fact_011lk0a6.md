---
id: fact_011lk0a6
type: fact
statement: H0.1 fechada: areas por ordem + flags globais (--json/--quiet/--log-level) + envelope JSON; make check verde
created_at: 2026-10-17T23:16:07.425Z
body_hash: 8aebbed8
schema_version: 2
anchors: [crates/ktu-cli/src/cli.rs]
---
H0.1 (task_003n4ebj) fechada com sucesso. Incrementos: (a) reordenacao do enum Command por AREA (CONVERSA run; CONHECIMENTO memory/guardrails/plugins/trash; PROJECTO init/doctor/mode; SISTEMA prime/self/providers; PORTA serve/tui) + rodape AFTER_HELP com as areas e o ciclo; (b) flags globais no Cli: --json, --quiet, --log-level (env KTU_LOG_LEVEL, default info), com #[arg(global = true)]; (c) logging::init(level) — quiet mapeia para nivel off; (d) novo crates/ktu-cli/src/output.rs: enum Format {Text,Json} + Success/Failure/ErrorInfo (serde) + emit/emit_error — o envelope {success,command,data?,error?}; prime e self version emitem envelope com --json; (e) lint fixes: Format em vez de bool (max-fn-params-bools=0), allow struct_excessive_bools em Cli (json+quiet) e struct_field_names em Success/Failure (chave success). Verificado: ktu --help mostra as areas; ktu --json prime/self version emitem envelope; ktu --quiet nao escreve em stderr; make check VERDE. H0.5 (task_012xuf1i) em progresso: stdin/heredoc feito; falta envelope nos restantes verbos + codigos de saida documentados.