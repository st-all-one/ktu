# Tool monitor / inspection

- **Estado:** candidata (do `goose`) — **não trazer** (ktu já cobre o essencial).
- **Origem:** `REF/goose/crates/goose/src/{tool_inspection,tool_monitor,tool_call_labels}.rs`;
  `wiki/revisao-goose.md` §2.8.
- **Âncora:** `crates/ktu-core/src/agent/logging.rs`, `crates/ktu-core/src/events.rs`.

## O que é
Monitorização/inspecção de chamadas de tool: rótulos, inspecção de argumentos/resultados,
painéis de acompanhamento em tempo real.

## Porque não entra
O ktu já emite **eventos estáveis** (`tool.call`, `tool.result`, `guard.decision`) pelo log
e pela porta `Logger`; a TUI/web consomem-nos. Um subsistema de monitorização dedicado é
redundante.

## Pré-condições para reconsiderar
- Um consumidor de UI que precise de agregação em tempo real (ex.: painel de tools).

## Custo e risco
- **Esforço:** médio.
- **Risco de não fazer:** baixo.
- **Risco de fazer (YAGNI):** duplicar a observabilidade existente.

## Decisão pendente
Manter fora. Se surgir uma UI que precise, **derivar** dos eventos já existentes.
