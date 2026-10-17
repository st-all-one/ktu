# Auditoria densa (store + bloom + índice)

- **Estado:** deferido.
- **Origem:** katu ADR `0009-auditoria-densa.md`, `audit/`.
- **Âncora:** `crates/ktu-core/src/agent/logging.rs`, `.ktu/sessions/*.jsonl`.

## O que é
Um subsistema de auditoria persistente: registo denso de decisões com **filtro de
Bloom** (evitar duplicados) e **índice** consultável, além do log de sessão. Serve para
responder "porque é que esta acção foi permitida/negada" a posteriori, com volume.

## O que o ktu tem hoje
- **Log de sessão append-only** (fonte da verdade) com eventos estáveis (`guard.decision`,
  `plugin.dispatch`, `tool.call/result`, `turn.start/end`).
- Logs estruturados via porta `Logger` (`stderr`), custo zero por omissão.
- `kd` (knudge) guarda a evolução do projecto com âncoras.

## Porque não entrou
O log de sessão + os eventos estruturados já dão a evidência necessária ao gate e à
depuração. Não há consumidor de consulta agregada de auditoria (nem volume que o exija).

## Pré-condições para reconsiderar
- Necessidade de **consultar** decisões de guardrail agregadas (ex.: relatório de
  conformidade), ou de retenção/consulta com volume que o JSONL de sessão não sirva.
- Um requisito de **não-repúdio** (auditoria como prova).

## Custo e risco
- **Esforço:** médio (store + bloom + índice + CLI de consulta).
- **Risco de não fazer:** baixo — a evidência existe, só não é agregada.
- **Risco de fazer (YAGNI):** subsistema de escrita no *hot path* sem consumidor.

## Decisão pendente
Só com um requisito concreto de consulta/prova. Se entrar, derivar do log (nunca uma
segunda fonte da verdade).
