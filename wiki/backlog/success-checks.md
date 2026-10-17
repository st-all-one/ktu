# Success checks

- **Estado:** candidata (do `goose`) — **avaliar**.
- **Origem:** `REF/goose/crates/goose/src/agents/retry.rs` (`SuccessCheck::Shell`);
  `wiki/revisao-goose-profunda.md` §1.14.
- **Âncora:** `crates/ktu-cli/src/run.rs`, `crates/ktu-core/src/domain.rs`.

## O que é
Após uma tarefa/run, corre comandos de verificação (`shell`) com **timeout**; falha → o
resultado **não** é sucesso. Um "gate" declarativo do resultado, dado pelo utilizador.

## Porque interessa
Dá ao utilizador uma forma de exigir *"só conta como feito se `make check` passar"* — **dado**
declarativo, alinhado com a cultura de gates do ktu (a fase só fecha com `make check` verde).
Fecha o turno com um `StopReason`/veredicto honesto em vez de "o modelo disse que acabou".

## O que o ktu tem hoje
`StopReason` (EndTurn/MaxSteps/Cancelled/Error/Stagnation); sem verificação de sucesso
declarativa.

## Pré-condições para reconsiderar
- Um comando de verificação canónico por projecto.

## Custo e risco
- **Esforço:** baixo-médio (lista de checks + timeout + veredicto no envelope).
- **Risco de não fazer:** "sucesso" sem prova.
- **Risco de fazer:** check lento/falso — timeout + opt-in; o veredicto é do check, não do
  modelo.

## Decisão pendente
Avaliar. Checks declarativos com timeout; o veredicto entra no envelope (dados, não stdout);
opt-in por projecto.
