# knudge — `health::doctor` (saúde da memória)

- **Estado:** candidata (do `knudge` 0.6.1) — **incorporar**.
- **Origem:** `crates/knudge/wiki/integration/08_saude_e_manutencao.md`,
  `crates/knudge/wiki/specs/saude.md`; `wiki/revisao-knudge.md` §6.1.
- **Âncora:** `crates/ktu-memory/src/report.rs`.

## O que é
`doctor` corre **13 verificações** (somente leitura) sobre o corpus: integridade do grafo,
ciclos, âncoras quebradas, duplicatas, arestas sugeridas em falta, locks stale, etc.
`doctor_fix` corrige `body_hash`, âncoras, locks e índice divergente — e é **idempotente**.
`audit` é leitura pura; `close_task` exige **evidência**.

## Porque interessa
O `ktu-memory` já tem um `health` próprio (contagens, linhas malformadas, âncoras quebradas) — mas
`doctor` é muito mais completo e **já testado**. Dá ao ktu um `ktu doctor --memory` real, com
reparo reversível.

## O que o ktu tem hoje
`MemoryHealth` (contagens + âncoras quebradas) em `report.rs`; sem checks nem `--fix`.

## Pré-condições para reconsiderar
- Nenhuma.

## Custo e risco
- **Esforço:** baixo (usar `health::{doctor,doctor_fix}`).
- **Risco de não fazer:** corpus degrada sem aviso.
- **Risco de fazer:** `--fix` a mexer no corpus — idempotente e só por acção explícita.

## Decisão pendente
Promover. `ktu doctor --memory` usa `doctor`/`doctor_fix`; o `--fix` é explícito e idempotente;
teste de corpus degradado.
