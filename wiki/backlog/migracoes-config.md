# Migrações de configuração

- **Estado:** **incorporada** (F5, gate G20) — `crates/ktu-core/src/config/migrate.rs`: `schema = 1`,
  `read_migrate` (leitura não destrutiva) + `migrate` (escrita idempotente), fail-closed em versão
  futura e `ktu config doctor`; decisão na [ADR 0020](../adr/0020-schema-de-config-versionado-com-leitura-nao-destrutiva.md).
- **Origem:** `REF/goose/crates/goose/src/config/migrations.rs`;
  `wiki/revisao-goose-profunda.md` §1.5.
- **Âncora:** `crates/ktu-cli/src/init.rs` (e o loader de config, se adotado — [76](./config-precedencia.md)).

## O que é
Migra o esquema da config (`run_migrations(&mut Mapping) -> bool`), com uma variante **não
destrutiva** (`run_read_migrations`) para caminhos de leitura em memória que não podem remover
chaves planas que outros chamadores ainda consultam.

## Porque interessa
Se o ktu adotar config (global/local), o esquema vai mudar. Uma migração explícita, idempotente
e **separada** da leitura evita corromper configs antigas — e o modo "só leitura" evita
migrações destrutivas num caminho de leitura.

## O que o ktu tem hoje
`init.rs` (bootstrap); sem config versionada nem migrações.

## Pré-condições para reconsiderar
- Adotar config em ficheiro (a migração só faz sentido aí).

## Custo e risco
- **Esforço:** baixo (função idempotente + versão de esquema).
- **Risco de não fazer:** configs antigas quebram.
- **Risco de fazer:** migração destrutiva na leitura — separar `read` de `write`.

## Decisão pendente
Promover (quando houver config). Migração idempotente com versão de esquema; variante
não-destrutiva para leitura; teste de config antiga.
