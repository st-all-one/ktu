# Editar em sessão / fork

- **Estado:** candidata (do `goose`) — **incorporar**.
- **Origem:** `REF/goose/documentation/docs/guides/sessions/in-session-actions.md`;
  `wiki/revisao-goose-profunda.md` §1.7.
- **Âncora:** `crates/ktu-cli/src/session_store.rs`, `crates/ktu-core/src/session/mod.rs`.

## O que é
Editar uma mensagem anterior e **truncar** tudo o que vem depois (*Edit in Place*), ou
**forkar** a sessão preservando a história até à mensagem editada (*Fork Session*). Na CLI:
`--resume <id> --edit` abre a conversa em YAML no `$EDITOR`.

## Porque interessa
Corrigir uma mensagem sem recomeçar é usabilidade objectiva; o **fork** é a versão simples da
árvore de sessão (pi, [40](./session-tree.md)). O log append-only do ktu suporta um fork por
**prefixo** sem reescrever história.

## O que o ktu tem hoje
Sessões persistidas em JSONL; sem edição/fork.

## Pré-condições para reconsiderar
- Necessidade de corrigir um pedido a meio.

## Custo e risco
- **Esforço:** médio (truncar = nova sessão por prefixo; fork = id novo).
- **Risco de não fazer:** recomeçar do zero.
- **Risco de fazer:** truncar contexto sem intenção — explícito, e o fork preserva o original.

## Decisão pendente
Promover. Truncar cria uma sessão nova a partir do prefixo (o log original fica); fork é
idêntico mas preserva a referência; nunca apagar história.
