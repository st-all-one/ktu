# Simulation (testes determinísticos end-to-end)

- **Estado:** candidata (do `opencode`) — **avaliar**.
- **Origem:** `REF/opencode/packages/simulation`; `wiki/revisao-opencode.md` §2.9.
- **Âncora:** `crates/ktu-core/src/testing.rs`, `crates/ktu-core/tests/loop.rs`.

## O que é
Um harness de **simulação** (backend/frontend, control-server, gravação) que exercita o
agente de forma **determinística**, sem LLM real: um guião produz eventos esperados.

## Porque interessa
O ktu preza determinismo (portas/fakes, `FakeProvider`/`FakeTools`, testes de ouro). Uma
**simulação end-to-end** (guião → eventos) é um upgrade dos testes actuais — cobre o loop
completo com sequências longas e reprodutíveis.

## O que o ktu tem hoje
`ktu_core::testing` (provider/tools falsos), `tests/loop.rs` (integração), `benches/core.rs`
(benchmark determinístico), bateria `quality_test` (com modelo real).

## Pré-condições para reconsiderar
- Sintomas de regressão que os testes unitários não apanham.
- Um formato de guião (entrada → eventos esperados) estável.

## Custo e risco
- **Esforço:** médio-alto (harness + formato de guião + runner).
- **Risco de não fazer:** baixo (os testes actuais cobrem o essencial).
- **Risco de fazer:** infra de teste a manter.

## Decisão pendente
Adiar. Se entrar, começar por um **guião simples** sobre `FakeProvider` (reaproveitar
`testing.rs`), não um subsistema completo.
