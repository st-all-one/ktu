# G20 — Migrações de config (schema, idempotência, fail-closed)

- **Veredicto:** ✅ **verde** (7/7) · **Artefactos:** `artifacts/G20/` · **Data:** 2026-10-18
- **Item:** [F5](../implementation_phase3/F5-migracoes-config.md) ·
  **Critério:** [GATES da Fase 3](../implementation_phase3/GATES.md) §G20.

## Cenário

Uma config **v0** (sem versão, com a forma plana/CSV de `deferred_tools`) nos **dois** níveis
(local e global): o diagnóstico, a leitura, a escrita (que migra) e uma config de versão **futura**
(v3) que tem de ser recusada sem tocar no ficheiro.

## Verificações

| # | Critério | Observado |
|---|---|---|
| 1 | o `doctor` mostra a versão dos **dois** níveis e se migrariam | 2 níveis em `schema=0`, `migrates=true` |
| 2 | o `doctor` **não** escreve | ficheiro com o hash de entrada |
| 3 | a leitura aceita a forma plana antiga **sem reescrever** o ficheiro | `bash,write_file` com o ficheiro intacto |
| 4 | a escrita migra com **aviso em `stderr`** (nunca em silêncio) | `migrada do schema v0 para v1` + `schema = 1` |
| 5 | migrar duas vezes **não muda nada** (idempotente) | hash `7909f243…` → `7909f243…`, `migrates=false` |
| 6 | versão futura ⇒ erro **legível** | `config no schema v3, mais recente que o v1 deste binario…` |
| 7 | a recusa **não escreve** | ficheiro com o hash de entrada |

## Artefactos

`doctor-v0.json` (o diagnóstico dos dois níveis) · `config-antes.toml` · `config-depois.toml`
(migrado) · `config-recusada.toml` (v3, intacto) · `hashes.txt` (antes/depois).

## Notas

- **Gatilho declarado vs. trabalho feito.** O gatilho do F5 («o schema mudar para v2 com configs
  v1 em uso») **não disparou**: nenhuma chave foi retirada desde o início (as `KEYS` nunca mudaram,
  verificado no histórico) e não há configs antigas em uso. O que se implementou é o **mecanismo**
  (versão + migração), que é pré-condição de qualquer migração futura — sem versão declarada não há
  base para migrar. Ver a nota completa na [ADR 0020](../adr/0020-schema-de-config-versionado-com-leitura-nao-destrutiva.md).
- **Desvio declarado do plano.** O passo 2 do plano previa que a variante de escrita pudesse
  **remover** chaves planas antigas. Não existe nenhuma chave obsoleta, e inventar uma seria uma
  migração fictícia: a lista está **vazia** e o mecanismo de retirada é exercitado no núcleo por um
  **passo de ensaio** (`apply_steps` com um passo sintético) — não por uma chave inventada na wiki.
- **As duas variantes são diferentes de propósito.** `read_migrate` normaliza em memória e **nunca
  retira** chaves (outro chamador pode consultá-las — a lição do `goose`); `migrate` escreve e pode
  retirar. A verificação 3 é a prova do contrato de leitura: o valor antigo é aceite e o ficheiro
  fica **byte-idêntico**.
- **`schema = 1` carimba-se na escrita**, não na leitura: uma leitura que reescrevesse o ficheiro
  surpreenderia (`ktu config get` a alterar o que inspecciona) e quebraria o «inspeccionar antes de
  mexer». Quem migra é a escrita; o `doctor` diz o que aconteceria sem tocar em nada.
- **Fail-closed em versão futura**: `schema = 3` num binário v1 é erro legível **e nenhuma
  escrita** — a única resposta segura para um ficheiro que não se sabe ler.
- **Fora do âmbito:** a config da memória (`memory.toml`) e o `auth.toml` não têm versão própria;
  se algum dia mudarem de schema, o mesmo mecanismo serve (a versão é do documento, não do
  ficheiro).
