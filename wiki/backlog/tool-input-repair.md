# Reparação de input de tool

- **Estado:** candidata (do `opencode`) — **incorporar** (consolida com [67](./tool-calls-declaradas.md)).
- **Origem:** `REF/opencode/packages/core/src/plugin/tool-input-repair.ts`;
  `wiki/revisao-opencode-profunda.md` §1.5.
- **Âncora:** `crates/ktu-core/src/agent/mod.rs`, `crates/ktu-provider/src/wire.rs`.

## O que é
Repara o input de tool calls quando o schema o suporta **inequivocamente**: objecto em string
(`'{"limit":"20"}'`), string numérica/booleana (`"20"`→`20`), `null`/`{}` para opcional, arrays
em string, tuplos posicionais, dicionários, campos aninhados e referências locais. Profundidade
máx. **6**; **só** quando é inequívoco.

## Porque interessa
Modelos fracos emitem argumentos em string; a reparação (limitada e verificada pelo schema)
salva turnos que hoje falham em `Decode`. É a **complementaridade** da declaração de tool calls
em texto ([67](./tool-calls-declaradas.md)): uma cobre a forma, a outra o tipo.

## O que o ktu tem hoje
Descodificação directa; sem reparação.

## Pré-condições para reconsiderar
- Modelo em uso que emita argumentos mal-tipados.

## Custo e risco
- **Esforço:** baixo-médio (reparação guiada pelo schema, profundidade limitada).
- **Risco de não fazer:** turnos perdidos em `Decode`.
- **Risco de fazer:** reparar errado — **só** quando inequívoco; caso contrário, falha fechado.

## Decisão pendente
Promover. Reparação guiada pelo schema, profundidade limitada, só inequívoca; falha fechado caso
contrário; teste por cada forma.
