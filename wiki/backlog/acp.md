# ACP (Agent Client Protocol)

- **Estado:** candidata (do `goose`) — **não trazer** (por omissão).
- **Origem:** `REF/goose/crates/goose/src/acp`, `goose-acp-macros`,
  `documentation/docs/gdk/acp`; `wiki/revisao-goose.md` §3.
- **Âncora:** `crates/ktu-port/src/protocol.rs`.

## O que é
Duas direcções: (a) o ktu **como servidor ACP** (editores como Zed/JetBrains ligam-se por
stdio); (b) **agentes ACP como provider** (delegar para Claude Code/Codex, que executam
as tools internamente).

## Porque não entra
O ktu tem a **sua porta** versionada (Channel/Http + SSE). ACP acrescentaria um protocolo
e um cliente/servidor sem consumidor. Como provider, seria um **segundo agente** a competir
com o loop do kernel (a mesma objecção do GDK).

## Pré-condições para reconsiderar
- Necessidade concreta de integrar com um **editor** (Zed/JetBrains).
- Ou delegar explicitamente para um agente ACP externo.

## Custo e risco
- **Esforço:** médio-alto (protocolo + transporte + sessão).
- **Risco de não fazer:** baixo.
- **Risco de fazer:** dois donos do loop; superfície nova.

## Decisão pendente
Manter fora. Reabre só com um cliente ACP real como requisito.
