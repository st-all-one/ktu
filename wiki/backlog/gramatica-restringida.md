# Gramática restringida (GBNF) / structured output

- **Estado:** deferido.
- **Origem:** katu ADR `0025-decodificacao-estruturada-por-schema.md`; `plan.md` §2.12-B.
- **Âncora:** `crates/ktu-provider/src/wire.rs`, `crates/ktu-provider/src/stream.rs`.

## O que é
Forçar a saída do `llama.cpp` a uma **gramática (GBNF)** derivada do schema das tools, de
modo que o *tool-call* JSON seja sempre válido — alavanca grande de qualidade para
modelos locais pequenos.

## O que o ktu tem hoje
- Tool-calling nativo (`--jinja`) + **fallback de parsing em texto**.
- Um modelo de referência (Ornith-1.5-9B) com tool-calling nativo fiável.
- `ktu providers doctor` faz uma sonda de tool-calling.

## Porque não entrou
O modelo de referência emite tool-calls válidos sem gramática; o fallback cobre o resto.
A gramática é uma optimização para modelos mais fracos.

## Pré-condições para reconsiderar
- Uso de **modelos pequenos/fracos** em tool-calling como alvo principal.
- Sintoma medido: JSON inválido recorrente apesar do fallback.

## Custo e risco
- **Esforço:** médio (gerar GBNF do schema; integrar no pedido; testes por família).
- **Risco de não fazer:** baixo com o modelo de referência.
- **Risco de fazer (YAGNI):** acoplamento a `llama.cpp` sem sintoma.

## Decisão pendente
Adiar. Se entrar, atrás de *flag* opt-in (não no caminho por omissão).
