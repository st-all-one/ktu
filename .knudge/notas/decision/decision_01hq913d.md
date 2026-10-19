---
id: decision_01hq913d
type: decision
statement: "Migracao de config: duas variantes (leitura nao destrutiva, escrita pode retirar), fail-closed e doctor read-only"
created_at: 2026-10-18T16:37:55.828Z
body_hash: 89682a8c
schema_version: 2
anchors: [crates/ktu-core/src/config/migrate.rs, wiki/adr/0020-schema-de-config-versionado-com-leitura-nao-destrutiva.md]
---
Decisao (F5, ADR 0020). A config declara a sua versao (schema=1); ausencia => v0. Duas variantes de migracao em vez de uma: read_migrate nunca retira chaves (a leitura nao sabe quem mais as consulta - licao do goose) e migrate pode retira-las na escrita. Alternativas descartadas: (a) uma so funcao - a leitura passaria a poder remover, que e exatamente o defeito que o plano queria evitar por construcao; (b) leitura tolerante a chaves desconhecidas (desligar deny_unknown_fields) - abriria mao da decisao do H2 (uma gralha numa chave deixaria de ser erro); (c) migrar automaticamente na leitura (reescrever ao ler) - `ktu config get` a alterar o ficheiro surpreende e quebra o inspeccionar-antes-de-mexer; (d) nao versionar e detectar a forma antiga por heuristica - sem versao declarada o v3 e indistinguivel de um erro de escrita; (e) inventar uma chave obsoleta para exercitar a retirada - migracao ficticia (a lista fica vazia, o mecanismo e testado com um passo de ensaio); (f) um `ktu config migrate` dedicado - superficie a mais, a escrita ja migra e o doctor diz o que aconteceria. Fail-closed em versao futura: erro legivel e nenhuma escrita. O gatilho declarado do F5 nao disparou: o que se implementou e o mecanismo (pre-condicao de qualquer migracao futura) - desvio registado na ADR.