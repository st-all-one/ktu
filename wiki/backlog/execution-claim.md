# Execution claim — recuperação de crash

- **Estado:** candidata (do `opencode`) — **incorporar**.
- **Origem:** `REF/opencode/specs/v2/session.md` (Execution Is Process-Local);
  `wiki/revisao-opencode-profunda.md` §1.3.
- **Âncora:** `crates/ktu-core/src/session/mod.rs`, `crates/ktu-core/src/store.rs`.

## O que é
Quando começa um período *busy* do processo, a execução **compromete um claim write-ahead**;
sucesso/falha/interrupção libertam-no, mas *shutdown* sujo **preserva-o**. No arranque, sessões
de topo com claim são **retomadas** do histórico durável, com uma **instrução de continuação** e
**contagem limitada de tentativas**; as tool calls pendentes são **falhadas** (*orphan
reconciliation*) antes de continuar. **Não** garante exactly-once.

## Porque interessa
O ktu tem retomada; um **claim** que sobrevive a um kill e **limita** as tentativas de
recuperação evita retomar em ciclo infinito ou deixar tool calls pendentes. É robustez real.

## O que o ktu tem hoje
Retomada + `checkpoint.rs`; sem claim write-ahead nem limite de tentativas.

## Pré-condições para reconsiderar
- Crash/kill a meio de um turno.

## Custo e risco
- **Esforço:** médio (claim + reconciliação + limite).
- **Risco de não fazer:** tool calls pendentes; retomada em ciclo.
- **Risco de fazer:** claim obsoleto — idade/limite de tentativas; reconciliação explícita.

## Decisão pendente
Promover. Claim write-ahead; no arranque, retomar com instrução de continuação e limite de
tentativas; falhar tool calls pendentes antes de continuar. Consolidar com
[64 — turn-lock](./turn-lock.md) e [63 — durabilidade-politica](./durabilidade-politica.md).
