# State machine do `goose-agent`

- **Estado:** candidata (do `goose`) — **não trazer**.
- **Origem:** `REF/goose/crates/goose-agent` (`Operation`/`Inference`/`EffectHandler`);
  `wiki/revisao-goose.md` §3.
- **Âncora:** `crates/ktu-core/src/agent/mod.rs`, `crates/ktu-core/src/agent/delegation.rs`.

## O que é
Uma máquina de estados genérica do agente, com passos (`Operation`/`Inference`), *effects*
e um `EffectHandler` plugável — separação limpa entre decisão e efeito.

## Porque não entra
O ktu **já tem** o seu loop com essa separação (kernel puro + portas + `Driver`/`Step`,
`StopReason`, `tool.result` num ponto único). Adoptar o `goose-agent` seria **reescrever**
o que passou o gate, por uma abstracção mais genérica (e mais pesada).

## Pré-condições para reconsiderar
- Um requisito de **múltiplos tipos de passo** (inferência, operação, sub-fluxos) que o
  loop actual não exprima.

## Custo e risco
- **Esforço:** alto (reescrita do loop + invariantes I1–I17).
- **Risco de não fazer:** nenhum (o loop funciona e passou o gate).
- **Risco de fazer:** perder os invariantes travados por teste.

## Decisão pendente
Manter fora. O loop do ktu é a tese; generalizá-lo não é prioridade.
