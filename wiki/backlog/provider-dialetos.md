# Providers finos por dialeto (Anthropic, …)

- **Estado:** deferido (alternativa ao GDK).
- **Origem:** `decision_00aw5etj` (alternativa documentada).
- **Âncora:** `crates/ktu-provider/src/`, `crates/ktu-provider/src/wire.rs`.

## O que é
Um **provider fino por dialeto** implementando a porta `Provider` do ktu directamente —
ex.: *Messages* da Anthropic, ou *Responses* da OpenAI — como o `opencode-go` já faz para
o OpenAI-compatible. Determinístico e testável com o *test double* HTTP existente.

## O que o ktu tem hoje
- `llama.cpp` built-in e `opencode-go` (OpenAI-compatible).
- `Flavor` de wire e um *test double* HTTP com fixtures (SSE fragmentado, tool-call
  nativa/texto, 429/timeout).

## Porque não entrou
Não há consumidor: os providers remotos usados falam OpenAI-compatible. Escrever dialetos
sem necessidade é o oposto do princípio (reaproveitar em vez de possuir).

## Pré-condições para reconsiderar
- Necessidade concreta de um provider **não** OpenAI-compatible (ex.: Anthropic).
- Fixtures reais do dialeto para o *test double*.

## Custo e risco
- **Esforço:** baixo-médio por dialeto (wire + parsing + fixtures).
- **Risco de não fazer:** bloqueio se surgir um provider específico.
- **Risco de fazer (YAGNI):** manter dialetos que ninguém usa.

## Decisão pendente
Promover quando surgir o 1.º provider não-OpenAI. Começar por Anthropic (*Messages*),
com fixtures e um teste de ouro do wire.
