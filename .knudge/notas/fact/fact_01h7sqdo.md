---
id: fact_01h7sqdo
type: fact
statement: "H5 concluida: edit multi-bloco/BOM, truncagem, remedy, eco, MaxSteps sem tools, tool calls declaradas"
created_at: 2026-10-18T00:32:26.657Z
body_hash: 3b028f2b
schema_version: 2
anchors: [crates/ktu-tools/src/text.rs, crates/ktu-tools/src/truncate.rs, crates/ktu-core/src/agent/toolcall.rs, crates/ktu-core/src/agent/mod.rs]
---
H5 (task_00htbspv, parcial) concluida. (1) edit multi-bloco atomico + BOM/line-endings: novo ktu-tools::text (Document detecta BOM/CRLF, aplica em LF, restaura; apply_edits aplica N blocos atomicamente, erro com o bloco). (2) Truncagem sistematica: ktu-tools::truncate (linhas+bytes, sem linhas parciais); bash guarda o output completo em .ktu/out/<tool>-<hash>.txt (FNV-1a) e devolve o ponteiro; read_file ganha offset (continuacao) e read.missing legivel. (3) Recusa acionavel: DenyReason::remedy() para todos os motivos; a mensagem de negacao para o modelo inclui o remedio. (4) Eco: StopReason::Echo (novo), nudge + 1 retry, nota visivel (nunca aceite); passo final de MaxSteps corre sem tools com pedido de resumo (nunca executa tools, mesmo se o modelo insistir). (5) Tool calls declaradas + reparacao: ktu-core::agent::toolcall decodifica chamadas em texto (<tool_call>, cerca, JSON) contra o catalogo fechado (nome fora -> ignorado; o texto declarado nao entra no log) e repara argumentos guiados pelo schema (profundidade <= 6, so inequivoco). Testes: +11 unit (text, truncate, toolcall) e +5 de loop (eco, controlo negativo, MaxSteps sem tools, declaradas). Adiado A6 (task_01qsu6vf): fila de mutacao por caminho (tools sequenciais na T2), remedio por regra, ancoras proximas no edit. Gate: make check + typos + machete verdes.