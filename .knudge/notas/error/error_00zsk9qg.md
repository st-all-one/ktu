---
id: error_00zsk9qg
type: error
statement: kd task update --parent nao movia a aresta results_in: pai antigo continuava a listar o filho
created_at: 2026-10-18T13:40:41.393Z
body_hash: b2ba84b9
schema_version: 2
anchors: [crates/knudge/crates/knudge-cli/src/commands/task/mutate.rs, crates/knudge/crates/knudge-core/src/task/mod.rs, crates/knudge/crates/knudge-core/src/graph/link.rs]
---
Defeito encontrado e corrigido na sessao (2026-10-18). O `reparent` (kd task update --parent) reescrevia APENAS o marcador <!-- knudge:parent --> no corpo do filho. A aresta `results_in` vive no frontmatter do PAI (task::attach -> graph::link) e nao era removida do pai antigo nem acrescentada ao novo. Consequencias observadas: (a) o pai antigo continuava a listar o filho; (b) `epico:` (derivado do grafo por epic_of) apontava para o epico errado; (c) o `progresso` dos dois epicos ficava errado (o antigo mostrou 5/11 com 5 filhos, todos fechados); (d) `kd task list --parent <novo>` nao mostrava os filhos movidos. Reproducao: criar epico E1, epico E2, issue I1 sob E1, issue I2 sob E2, tarefa T sob I1; `kd task update --id T --parent I2`; `kd task show --id T` -> pai: I2 mas epico: E1. Correcao: graph::unlink (novo) + task::attach/detach publicos; o `reparent` desanexa o filho de TODOS os pais que o listam no grafo (nao so do marcador, o que repara um estado inconsistente anterior) e anexa ao novo. Travado por knudge-cli/tests/reparent.rs. Nota: os dados ja quebrados nao se reparavam so com `kd doctor --fix` (o check `derived` considerava o indice coerente) - a reparacao veio de repetir o `update --parent` com o binario corrigido.