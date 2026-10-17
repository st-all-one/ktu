# `settings.json` com JSON Schema + diagnóstico

- **Estado:** candidata (do `pi`) — **incorporar** (condicionada à config, [76](./config-precedencia.md)).
- **Origem:** `REF/pi/packages/coding-agent/src/core/{settings-schema,settings-diagnostics}.ts`;
  `wiki/revisao-pi-profunda.md` §1.7.
- **Âncora:** `crates/ktu-cli/src/init.rs` (config, se adotada).

## O que é
A config tem um **JSON Schema** (autocompletação e validação no editor, via `"$schema"`) e um
**diagnóstico** que valida chaves/valores e reporta erros com sugestões.

## Porque interessa
Se o ktu adotar config em ficheiro, um schema + diagnóstico tornam-na **auto-documentada** e
validada — alinha com o conjunto fechado de chaves (`deny_unknown_fields`) do katu
([76](./config-precedencia.md)) e evita configs silenciosamente inválidas.

## O que o ktu tem hoje
Sem config em ficheiro (só `.agents/`).

## Pré-condições para reconsiderar
- Adotar config em ficheiro (o schema só faz sentido aí).

## Custo e risco
- **Esforço:** baixo-médio (schema gerado + diagnóstico).
- **Risco de não fazer:** configs inválidas silenciosas.
- **Risco de fazer:** schema a divergir do código — gerar do tipo, não à mão.

## Decisão pendente
Promover (com a config). Schema **gerado** do tipo de config; diagnóstico com lista + sugestão;
teste de chave desconhecida e de valor inválido.
