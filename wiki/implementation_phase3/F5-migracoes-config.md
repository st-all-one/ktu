# F5 — Migrações de configuração

> **Estado:** ✅ **fechado** (gate [G20](../gates/G20-migracao.md) verde 7/7, 2026-10-18) —
> **sem o gatilho declarado** (ver «Desvio» abaixo).
> **Issue:** `task_00g0us7a` · **Âncora:**
> [migracoes-config](../backlog/migracoes-config.md) · **Pré-requisitos:** H2 ·
> **Gatilho:** o schema mudar para **v2** com configs v1 em uso.

## Objetivo

Versionar o schema da config e migrar de forma **idempotente**, com uma variante **não destrutiva**
no caminho de leitura (o *loader* não pode remover chaves que outro chamador ainda consulta).

## Estado verificado (2026-10-18)

- `crates/ktu-core/src/config.rs` **não tem** campo de versão (nem `schema`, nem `version`).
- Não há configs antigas em uso ⇒ **nada a migrar hoje**. Este item é um **gatilho**, não pressa.

## Passos

1. `version = 1` (ou `schema = 1`) em `.agents/config.toml` e `~/.config/ktu/config.toml`;
   ausência ⇒ **v0** (implícito — o formato actual).
2. `migrate(&mut Mapping) -> bool` (escrita; pode remover chaves planas antigas) e
   `read_migrate(&Mapping) -> Mapping` (leitura; só acrescenta o que falta, **nunca** remove).
3. Idempotência: correr duas vezes não muda nada (hash igual).
4. Versão **desconhecida** (v3 num binário v2) ⇒ erro legível e **fail-closed** (não corromper).
5. `ktu config doctor` mostra a versão e se migraria (sem escrever).

## Tarefas kd

| Tarefa | Entrega |
|---|---|
| `task_015a04hb` | Versão de schema + migração (leitura não-destrutiva) |

## Resultado

| Passo | Entrega | Onde |
|---|---|---|
| 1 | `schema = 1`; ausência ⇒ v0; escrever **carimba** a versão | `crates/ktu-core/src/config/migrate.rs`, `crates/ktu-cli/src/config_cmd.rs` |
| 2 | `read_migrate` (leitura, nunca retira) + `migrate` (escrita, pode retirar) | `config/migrate.rs` (o v1 normaliza `deferred_tools`: CSV → lista) |
| 3 | idempotência (hash igual em duas passagens) | testes do núcleo + verificação 5 do G20 |
| 4 | versão desconhecida ⇒ erro legível e nenhuma escrita | `check_version` (fail-closed) |
| 5 | `ktu config doctor` (versão + migraria, **sem escrever**) | `crates/ktu-cli/src/config_doctor.rs` |
| — | gate | `xtask/src/gates/migration.rs` (G20, 7/7) |

## Desvio declarado (gatilho e passo 2)

- **O gatilho não disparou.** O plano di-lo no próprio texto («Não há configs antigas em uso ⇒
  nada a migrar hoje») e o histórico confirma: as `KEYS` nunca mudaram, não há chave obsoleta. O
  que se implementou é o **mecanismo** (versão + migração), que é pré-condição de qualquer migração
  futura — sem versão declarada não há base para migrar. Registado na
  [ADR 0020](../adr/0020-schema-de-config-versionado-com-leitura-nao-destrutiva.md).
- **O passo 2 ficou meio.** Dizia que a variante de escrita podia **remover** chaves planas
  antigas; não existe nenhuma, e inventá-la seria uma migração fictícia. A lista de obsoletas está
  **vazia** e o mecanismo de retirada é exercitado no núcleo por um **passo de ensaio**
  (`apply_steps` com um passo sintético).
- **O v1 normaliza `deferred_tools`** (string CSV → lista): era o único ponto onde uma forma
  plana antiga **falhava** a leitura (`ktu config set deferred_tools a,b` grava lista, mas um
  ficheiro escrito à mão com a string era erro). Achado registado como **A5** em
  [`gates/README.md`](../gates/README.md) §5.

## Gate / testes

- **G20** — ver [GATES](./GATES.md).
- v0 → v1 migra; correr 2× ⇒ mesmo resultado (idempotente).
- `read_migrate` **não** remove chaves (teste com uma chave plana antiga).
- v3 ⇒ erro legível e **nenhuma** escrita.

## Riscos

- Migração destrutiva no caminho de leitura → a variante `read_migrate` é o antídoto (lição do
  `goose`).
- Migração silenciosa → toda a migração escreve um aviso em `stderr` e fica logada.
