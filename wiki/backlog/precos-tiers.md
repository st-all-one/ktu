# Preços/tiers completos

- **Estado:** deferido.
- **Origem:** katu `policy/prices.toml`, `policy/tiers.toml`, ADR `0012`.
- **Âncora:** `crates/ktu-core/src/budget.rs`.

## O que é
Um **registo versionado** de preços por modelo e um **roteador por tier** (fast/balanced/
deep) ligado à fase do trabalho, para controlar custo/latência e escolher modelo.

## O que o ktu tem hoje
- `BudgetGate`: consumo (passos, tokens estimados, tempo) com **um dono do tecto**;
  atingir o tecto é recusa, nunca cortar evidência.
- Um modelo por sessão (`--model`), sem roteamento por fase.

## Porque não entrou
- Os providers actuais são local (`llama-server`, sem preço) e `opencode-go` (um modelo
  por sessão). Não há custo variável a gerir nem catálogo multi-modelo.
- Tiers por fase exigem fases (`Event::PhaseTransition`) que o loop não emite.

## Pré-condições para reconsiderar
- **Vários** providers/modelos com preços diferentes em uso simultâneo.
- Um requisito de orçamento em **dinheiro** (não só tokens/passos).

## Custo e risco
- **Esforço:** médio (catálogo + roteador + validação de esquema).
- **Risco de não fazer:** baixo com 1–2 modelos.
- **Risco de fazer (YAGNI):** dado de preços a envelhecer sem consumidor.

## Decisão pendente
Adiar até haver ≥2 modelos com custo real. O tecto de tokens/passos já protege o loop.
