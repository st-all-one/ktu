# Modo adversário (revisor independente)

- **Estado:** candidata (do `goose`) — **avaliar**.
- **Origem:** `REF/goose/documentation/docs/guides/security/adversary-mode.md`;
  `wiki/revisao-goose-profunda.md` §1.8.
- **Âncora:** `crates/ktu-core/src/guard.rs`, `crates/ktu-core/src/agent/mod.rs`.

## O que é
Um **revisor silencioso** que avalia cada tool call contra regras em linguagem natural
(`adversary.md`), devolvendo **ALLOW**/**BLOCK**; bloqueado não repete; se o revisor falha,
**fail-open**. Usa o mesmo modelo (sem chave nova); por omissão revê o `shell`.

## Porque interessa
É um "segundo par de olhos" contextual que os guardrails determinísticos não dão: percebe a
intenção ("isto faz sentido para o pedido original?").

## O que o ktu tem hoje
Guardrails determinísticos + `Guard`; sem revisor contextual.

## Pré-condições para reconsiderar
- Ameaças que os padrões não apanham.

## Custo e risco
- **Esforço:** médio (LLM por tool call + parsing ALLOW/BLOCK).
- **Risco de não fazer:** ataques contextuais passam.
- **Risco de fazer:** **custo por tool call**, **não-determinismo** e **fail-open** (contra o
  fail-closed do ktu) — só opt-in, com consciência do custo; preferir determinismo.

## Decisão pendente
Adiar. Se entrar, **opt-in**, revê só tools de risco, e o resultado é **consultivo** (nunca
substitui o guard determinístico); registar a decisão e o custo.
