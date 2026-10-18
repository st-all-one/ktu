---
id: fact_01lw86u2
type: fact
statement: "Comandos do projecto (.agents/commands/*.md) invocaveis por /nome args no REPL, com $ARGUMENTS e frontmatter partilhado"
created_at: 2026-10-18T02:58:48.623Z
body_hash: cbb7d5a8
schema_version: 2
anchors: [crates/ktu-core/src/command.rs, crates/ktu-core/src/frontmatter.rs, crates/ktu-cli/src/repl.rs, wiki/convencoes.md]
---
H10: novo ktu-core/src/command.rs - Command{name,description,prompt,path}; parse(file,text) aceita frontmatter opcional (sem ele, o corpo todo e o prompt e o nome vem do ficheiro), valida o nome (sem espacos nem /) e devolve Ok(None) se nao houver prompt; render(args) substitui $ARGUMENTS ou acrescenta os argumentos no fim; load(fs,dir) e deterministico e FAIL-OPEN (devolve (comandos, problemas) - ficheiro malformado nao impede a sessao, e reportado como aviso); find(commands, name) resolve por nome exacto. O REPL (ktu session) carrega .agents/commands/ no arranque, resolve /nome args para Slash::Custom{name,args} e entrega o prompt renderizado ao turno normal (M4: o comando nao tem logica propria); /help lista os comandos do projecto; desconhecido = erro accionavel. Novo verbo ktu commands ls (texto <nome>\\t<descricao>, --json com name/description/path/problems; problemas para stderr via report_error - print_stderr e negado). Refactor: o parser de frontmatter (chave: valor escalar) saiu de guardrails.rs para ktu-core/src/frontmatter.rs (split/parse/body/unquote/required) e os guardrails passam a usa-lo - um parser so para guardrails, comandos e agentes. Testes: command.rs (5 unit), frontmatter.rs (5 unit), ktu-cli/tests/commands.rs (4). Doc: wiki/convencoes.md §4.