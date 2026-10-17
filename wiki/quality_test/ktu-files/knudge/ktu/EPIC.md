# EPIC — Reimplementar o knudge em TypeScript

> **Documento-fonte:** [`knudge.md`](knudge.md) (síntese técnica do snapshot `REF/knudge` v0.5.4).
> **Objetivo deste épico:** um `kd`/`knudge-mcp` em TypeScript que produza **o mesmo contrato de
> bytes** sobre o mesmo repositório de memória, validado por um **oráculo diferencial** contra o
> binário Rust.
> **Estado do documento:** proposta executável (plano mestre + 22 issues em cadeia sequencial).
> **Todas as medições citadas** foram feitas nesta sessão nesta máquina (AMD Ryzen 5 5500U, Linux
> 7.2.10, Rust 1.99.0, Node 22.23.2, Bun 1.4.2, Deno 2.9.5, `kd` 0.6.1); comandos no Anexo A.

---

## 0. Sumário executivo

**Veredicto: é viável, com fidelidade total em `notas/` e três divergências declaradas.**

O porte é defensável porque o knudge guarda o estado em **ficheiros de texto com contrato de
bytes** (§2 de `knudge.md`) e não numa base de dados opaca: dois programas diferentes podem escrever
o mesmo repositório e ser comparados campo a campo. Duas medições desta sessão removem os dois
receios clássicos:

| Receio | Medição | Veredicto |
|---|---|---|
| "JS não reproduz o `f64` do Rust" | 37/37 sondas **bit-idênticas** (`log1p`, `log`, `log2`, `sqrt`, `exp`, `pow`); 4/14 divergências **só na formatação número→texto** | **Falso alarme numérico**; o risco real é *formatação*, não aritmética |
| "não há `flock` em Node" | o lock do knudge é um ficheiro criado com `O_CREAT\|O_EXCL` (`.knudge/.locks/<alvo>.lock`, `{"at": ms}`, stale 30 s, 50 tentativas, jitter 20 ms) | **Portável 1:1** com `fs.openSync(path,'wx')` → Rust e TS **coexistem** no mesmo repo |
| "arranque do Node mata a CLI" | piso medido: `kd --help` **7,5 ms**, `node -e ""` **33,2 ms**, **`bun -e ""` 7,2 ms** | Node penaliza; **Bun empata com o Rust** → o runtime de distribuição é uma decisão de projeto, não um destino |

O que se **perde** e não é recuperável: as garantias *mecânicas* do Rust (`forbid(unsafe_code)`,
`clippy::unwrap_used`/`panic`/`indexing_slicing`, gate de 300 linhas imposto por script, Miri) passam
a ser **convenções verificáveis por linter**, e o par V8/ICU introduz um acoplamento de *versão* que
o Rust não tem (a mesma entrada pode formatar diferente entre versões de Node). Recomendação:
**TS como runtime de produto** (distribuição npm, MCP, IDE, reutilização no browser) e **Rust como
oráculo de conformidade** em CI — não como substituto silencioso.

---

## 1. Objetivo e não-objetivos

**Objetivo.** Um único pacote TS com: biblioteca de domínio pura (`@knudge/core`), CLI `kd` com os
mesmos 14 verbos, servidor MCP com os 4 gatilhos, contrato JSON/exit codes idêntico, e `notas/**`
byte-a-byte igual ao Rust para a mesma sequência de operações.

**Não-objetivos.** (a) Redesenhar o domínio, o schema, o TOON ou os algoritmos — o porte é
**fidelidade**, e qualquer melhoria entra como decisão explícita depois de C1/C3 verdes; (b)
introduzir daemon, base de dados, servidor remoto ou async no núcleo; (c) ganhar desempenho; (d)
substituir o Rust no repositório original durante este épico.

---

## 2. Contrato de aceitação (o que significa "igual")

O oráculo é o binário Rust. Sete níveis, com exigência decrescente:

| Nível | Superfície | Exigência | Porquê |
|---|---|---|---|
| **C1** | `notas/**/*.md` | **Byte-a-byte igual** | é a verdade versionada; sem isto não há porte |
| **C2** | envelope `--json` + exit codes + `stdout`/`stderr` | **Byte-a-byte igual** | contrato de máquina (§5) |
| **C3** | `kd prime`, `prime --long` | **Byte-a-byte igual** (golden) | protocolo estático por versão |
| **C4** | `kd rewind` / `--resume` | Byte-a-byte **quando o relógio é o mesmo** (`Clock` injetável) | dinâmico por natureza |
| **C5** | `eventos/events.jsonl` | Byte-a-byte quando possível, senão igualdade de campos + ordem | auditoria; `merge=union` tolera |
| **C6** | `.idx/**` (derivado) | **Não exige bytes iguais** — exige *mesmos top-k* para as mesmas consultas | é descartável por desenho (§3); byte-equality não paga o custo |
| **C7** | propostas de dedup/MinHash-LSH | **Recall ≥** e conjunto de propostas *idêntico ou superconjunto* | LSH é aproximado (D204); igualdade exata não é garantível |
| **C8** | aritmética de score/ordenação | **tolerância 0** (comparação de bits) | ordem e tie-breaks são contrato (§4) |

Duas regras de convivência obrigatórias:

- **Namespace do derivado (ADR-TS-07).** O TS grava `index_format` próprio (ex.: `retrieval-v4-ts`)
  e o `doctor` reporta a implementação. Sem isso, Rust e TS apagam/regeneram o `.idx/` um do outro.
- **Nunca tocar `notas/` sem lock compatível.** O lock é o mesmo ficheiro/Nome/semântica (§0), pelo
  que a convivência é segura *desde a primeira linha do store* (issue I-07).

---

## 3. Evidência de viabilidade (medida nesta sessão)

**(a) Piso de arranque de processo** — mediana de 15 execuções, `µs`:

| Comando | min | mediana | p95 |
|---|---:|---:|---:|
| `kd --help` (Rust) | 5 897 | **7 540** | 7 829 |
| `kd self version` (Rust) | 4 271 | **7 429** | 8 296 |
| `node -e ""` | 25 905 | **33 215** | 52 136 |
| `node` + `fs`+`crypto`+`path` | 28 528 | **38 043** | 44 263 |
| **`bun -e ""`** | 4 233 | **7 230** | 14 163 |
| `deno eval "" --no-config` | 26 515 | **28 657** | 467 356 |

→ Node custa **+26 ms por invocação** face ao Rust; Bun **empata**. O piso fixo do Rust (~4–13 ms,
§6.2 de `knudge.md`) é justamente o custo que domina os verbos baratos, pelo que esta escolha decide
a ergonomia da CLI.

**(b) Fidelidade numérica Rust 1.99 vs V8 (Node 22)** — 51 sondas, comparação de bits IEEE-754:

| Família | Probes | Divergências |
|---|---:|---:|
| `ln_1p` (IDF de BM25), `ln` (JS-divergence), `log2` (nDCG), `sqrt` (Wilson), `exp`, `pow` (`0.5^(t/ttl)` do FSRS) | **37** | **0** |
| formatação número→texto (`{}` do Rust vs `String()` do JS) | 14 | **4** |

Divergências de formatação (todas reais e todas evitáveis escrevendo o emissor à mão):

| valor | Rust `{}` | JS `String()` |
|---|---|---|
| `1e-7` | `0.0000001` | `1e-7` |
| `1e-10` | `0.0000000001` | `1e-10` |
| `1e21` | `1000000000000000000000` | `1e+21` |
| `-0.0` | `-0` | `0` |

O Rust nunca usa notação científica em `{}`; o JS muda de notação em `1e-7`/`1e21` e perde o sinal
do zero negativo. Como o TOON **é** o contrato (§2 de `knudge.md`), isto obriga a um `formatF64`
próprio — é o item de risco nº 1 do porte.

**(c) Data path** — listar + ler + dividir frontmatter + contar chaves + somar bytes, **1000 notas**
(2,35 MB, 9 chaves/nota), mediana de 11 execuções:

| Runtime | mediana (processo completo) | dentro do processo | por nota |
|---|---:|---:|---:|
| Rust | **20,3 ms** (min 11,5) | 11,8 ms | **11,8 µs** |
| Bun 1.4.2 | **40,9 ms** (min 31,0) | ~34 ms | ~34 µs |
| Node 22 | **74,4 ms** (min 64,7) | ~41 ms | 33,5 µs |

As três produziram o **mesmo resultado** (`chaves=9000 bytes=2346790 soma=215264978`) — isto é, um
teste diferencial de semântica já funciona no primeiro dia. Trabalho por nota: **~2,8× o Rust**.

**(d) Mecanismos do SO** — lido no fonte `knudge-core`: lock por criação exclusiva de ficheiro com
conteúdo `{"at": <ms>}`, `stale_ms = 30_000`, `retries = 50`, `jitter_ms = 20`, reclaim por *rename*
para sidecar `.stale`, ordem externo-antes-de-interno (anti-ABBA). Tudo reproduzível com primitivas
do `node:fs`; nada depende de `flock`/`fcntl` nativos.

---

## 4. Vantagens do porte

1. **Distribuição e adoção.** `npm i -g`/`npx` num comando, sem toolchain Rust para contribuidores;
   o público-alvo (agentes, IDEs, automações) é JS-first.
2. **Superfície MCP nativa.** O ecossistema MCP é JavaScript; o `knudge-mcp` deixa de ser um binário
   extra e passa a poder ser importado/embutido (D101 deixa de ser um problema).
3. **Reutilização de código fora do terminal.** O mesmo parser TOON/schema corre no browser → UI de
   revisão de memória, extensão de IDE, sem WASM nem FFI.
4. **Embeddings triviais.** `fetch`/`undici` substituem o cliente HTTP/1.1 bloqueante escrito sobre
   `std::net` (§4.6 de `knudge.md`), incluindo *streaming* e *timeouts*.
5. **Fidelidade numérica comprovada.** 0 divergências em 37 sondas (§3b): BM25, RRF, FSRS, Wilson e
   JS-divergence migram sem ajuste.
6. **Arranque competitivo com o runtime certo.** Bun mede 7,2 ms (§3a) — a desvantagem histórica do
   JS desaparece, e `bun build --compile`/Node SEA dão binário único por plataforma.
7. **Iteração e modelação de dados evolutiva.** O schema cresce por extensões aditivas (§2:
   `claims`/`provenance`); em TS, validação estrutural e tipos condicionais acompanham isso com menos
   cerimónia do que o `Value` fechado do Rust.
8. **Onboarding de agentes.** O próprio knudge é usado por agentes que leem código; TS com tipos
   `readonly` e esquema explícito é um alvo mais fácil de manter por LLMs do que 56k linhas de Rust
   com portas genéricas.

---

## 5. Desvantagens e riscos (com mitigação e critério de abandono)

| # | Desvantagem | Gravidade | Mitigação |
|---|---|---|---|
| D1 | **2,8× mais lento no data path** (33,5 µs vs 11,8 µs/nota); end-to-end 2–3,7× | média | medir a 10⁴/10⁵ notas (I-22); aceitar no piso do corpus real; Bun como runtime de distribuição |
| D2 | **Sem garantias mecânicas** (`unsafe`/`clippy`/Miri/300 linhas nativas) → regras passam a convenção | alta | ADR-TS-05: `tsc --noEmit` estrito + eslint proibindo `any`/`as`/`!`/`Date.now`/`process.env`/`Math.random`/`for..in`; gate `max-lines: 300`; revisão obrigatória |
| D3 | **Acoplamento a V8/ICU**: `normalize`, `Math.*`, formatação podem mudar entre versões | alta | *pin* de Node/Bun + goldens byte-exatos (C1/C3) como teste de versão; `engines` no `package.json` |
| D4 | **Formatação f64 divergente** (§3b: `1e-7`, `1e+21`, `-0`) | alta | `formatF64` próprio + teste diferencial com a lista de bordas do anexo B de `knudge.md` (I-05) |
| D5 | **Ordenação permissiva**: `Object.keys` reordena chaves inteiras, `Map` é ordem de inserção, `sort` é estável | alta | ADR-TS-06: nunca iterar objetos do domínio; comparadores **totais** com desempate final por `id` (igual aos `total_cmp` do Rust) |
| D6 | **`BigInt` na MinHash/LSH** (bitwise JS é 32-bit) | média | `BigInt.asUintN(64, …)`; medir custo do `doctor` denso (I-14); se pesar, LSH em `Uint32Array` com dupla palavra |
| D7 | **Contágio `async`** (a API passa a `Promise`) | média | ADR-TS-03: núcleo puro síncrono; `async` só nos adaptadores (`Embedder`, `drain`) |
| D8 | **Supply chain npm** (superfície muito maior que 8 crates) | média | ADR-TS-02: **zero deps de runtime**; só `devDependencies`; lockfile + `--ignore-scripts`; auditoria no CI |
| D9 | **Fuzz/proptest menos rigoroso** para um parser de bytes | média | fast-check + fuzz diferencial contra o oráculo Rust (I-03/I-05); os corpus de fuzz do repo são entradas do teste |
| D10 | **Memória** (índices como objetos JS) e pressão de GC em `doctor` denso | baixa | estruturas compactas (`Int32Array`/arrays paralelos) onde o perfil mostrar |
| D11 | **Duas implementações = dois bugs possíveis** | alta | o oráculo é o Rust; divergência **bloqueia** o merge até ser explicada por escrito em `DIVERGENCES.md` do projeto TS |

**Critérios de abandono (kill criteria)** — se algum falhar, o épico para e o Rust fica como motor:

- **K1 (fim de F1):** C1 e C3 não atingem 100 % no corpus golden → parar.
- **K2 (fim de F4):** os top-k de `ask` divergem do Rust nalguma consulta do corpus de qualidade →
  parar (é sinal de que a semântica de ordenação não é reproduzível com tolerância 0).
- **K3 (fim de F5):** o conjunto de propostas de dedup não é subconjunto/superconjunto válido do Rust
  em ≥ 99 % dos casos → parar.
- **K4 (fim de F8):** piso de arranque > 15 ms na distribuição escolhida → rebaixar para "MCP-only"
  (renunciar à CLI), que era o objetivo parcialmente.

---

## 6. Decisões de arquitetura do porte

| ID | Decisão | Justificação |
|---|---|---|
| **ADR-TS-01** | **Node 22 LTS** como baseline de desenvolvimento/CI + **Bun** (ou SEA) para o binário distribuído; **sem daemon** | §3a: só assim o piso de arranque fica na ordem do Rust sem violar o desenho "sem servidor" |
| **ADR-TS-02** | **Zero dependências de runtime**; `typescript`, `vitest`, `fast-check`, `esbuild`, `eslint` como dev | D8: um projeto cujo valor é um contrato de bytes não deve ter 300 pacotes transitivos |
| **ADR-TS-03** | Domínio **puro e síncrono**; `async` apenas nos adaptadores | traduz R16/R43 (o "sem async no núcleo" do Rust) para TS |
| **ADR-TS-04** | Dados `readonly`, funções puras, sem classes mutáveis; `Result`-like (`{ ok: true, value } \| { ok: false, error }`) em vez de `throw` no domínio | espelha o domínio puro + `Error`/`ErrorKind` do Rust sem exceções a atravessar camadas |
| **ADR-TS-05** | `strict` + `noUncheckedIndexedAccess` + `exactOptionalPropertyTypes` + 12 proibições de lint (§D2) + `max-lines: 300` | único substituto disponível para `clippy -D warnings` + gate de ficheiro |
| **ADR-TS-06** | Ordem canónica explícita em todo o domínio; proibido `for..in`/`Object.keys` sobre dados; comparadores totais com desempate por `id` | D5; espelha "`HashMap` proibido" (§2/§3 de `knudge.md`) |
| **ADR-TS-07** | `.idx/` com `index_format` próprio (`retrieval-v4-ts`) e `doctor` a reportar a implementação | C6: coexistência sem thrash |
| **ADR-TS-08** | `prime`/`rewind` são **gerados**, nunca escritos à mão; goldens do repo Rust copiados como fixtures | C3/C4: é o teste de versão de V8/ICU |

---

## 7. Plano mestre

Oito fases em **cadeia estrita** (cada gate é o critério para abrir a fase seguinte). O objetivo de
cada fase é "mais uma camada de contrato a passar no oráculo", nunca "mais features".

| Fase | Nome | Issues | Gate de saída (verificável) |
|---|---|---|---|
| **F0** | Fundação e oráculo diferencial | I-01…I-02 | `pnpm check` verde (strict+lint+300 linhas); o oráculo corre Rust e TS e compara `notas/` |
| **F1** | Contrato de bytes | I-03…I-05 | **C1** e **C3** a 100 % no corpus golden; C8 nas sondas numéricas |
| **F2** | Portas, SO e persistência | I-06…I-09 | `init` byte-a-byte igual; escrita atómica + lock interoperável (teste com Rust a escrever em paralelo) |
| **F3** | Índice e recuperação lexical | I-10…I-11 | BM25 bit-idêntico para todo o corpus; `.idx` reconstruível e semânticamente equivalente (C6) |
| **F4** | Fusão, rank e `ask` | I-12 | **K2** passa: top-k idêntico em 100 % das consultas do corpus de qualidade |
| **F5** | Escrita, dedup e ciclo de vida | I-13…I-15 | C1 mantém-se sob `write`/`update`/`forget`; **K3** passa; `doctor` sem divergências |
| **F6** | Grafo e inferência | I-16 | PPR/Louvain/TMS determinísticos e idênticos ao Rust (mesmos scores, mesmos ids) |
| **F7** | Tarefas, fluxo e manutenção | I-17 | `task`/`sync`/`maintenance prune` byte-a-byte em `notas/`; C5 nos eventos |
| **F8** | Embeddings e auto-drain | I-18 | modos `none`/`lightweight` equivalentes; **K4** (piso ≤ 15 ms) avaliado |
| **F9** | CLI completa e MCP | I-19…I-20 | C2 a 100 % (envelopes/exit codes); `prime`/`rewind` goldens; 4 gatilhos MCP com os mesmos *hints* |
| **F10** | Distribuição e auditoria final | I-21…I-22 | binário por plataforma + npm; relatório diferencial final e `DIVERGENCES.md` do TS fechado |

**Estimativa de esforço** (1 dev sénior; calibração grosseira, para ordenar, não para prometer):

| Fase | F0 | F1 | F2 | F3 | F4 | F5 | F6 | F7 | F8 | F9 | F10 |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| dias | 4 | 8 | 7 | 9 | 6 | 12 | 8 | 6 | 5 | 7 | 6 |

Total ≈ **78 dias-dev** (≈ 4 meses de uma pessoa). Ordem de corte se o tempo apertar: F8 (embeddings
→ modo `none`), depois F6 (grafo → só consulta), depois F9-MCP (fica só a CLI), depois F7 (`task`).
**Nunca** cortar F1/F2/F4: são o contrato.

---

## 8. Grafo das issues (cadeia sequencial)

```
F0  I-01 repo+toolchain+gate ─▶ I-02 oráculo diferencial
F1                        ─▶ I-03 TOON codec ─▶ I-04 schema/nota/ids/hashes ─▶ I-05 formatF64+unicode
F2                        ─▶ I-06 portas+adaptadores+fakes ─▶ I-07 store+locks+atómico ─▶ I-08 eventos+doctor base ─▶ I-09 init+config+git
F3                        ─▶ I-10 tokenização+stemmer+índice ─▶ I-11 BM25
F4                        ─▶ I-12 canais+RRF+rank+ask
F5                        ─▶ I-13 lifecycle+dedup limiares ─▶ I-14 MinHash/LSH BigInt+merges ─▶ I-15 write/update/forget/batch
F6                        ─▶ I-16 grafo+TMS+ontologia+PPR/Louvain ─▶ (map)
F7                        ─▶ I-17 tarefas+fluxo+Wilson/FSRS/drift+manutenção
F8                        ─▶ I-18 embeddings+auto-drain
F9                        ─▶ I-19 CLI 14 verbos+prime/rewind ─▶ I-20 MCP 4 gatilhos
F10                       ─▶ I-21 empacotamento+distribuição+CI ─▶ I-22 auditoria final+bench+coexistência
```

Cada issue **tem uma única antecessora** (cadeia serial, como pedido). Paralelismo possível sem
quebrar o contrato: I-03/I-05 (fixtures), I-16/I-17 (independentes depois de I-15) e I-21 (empacotar
enquanto I-20 fecha). Fora esses três atalhos, a ordem é obrigatória: cada gate valida a fase.
---

## 9. Issues

> Convenções: **Dep.** = issue antecessora. **Tam.** = S (≤2 d) · M (3–5 d) · L (6–9 d) · XL (≥10 d).
> **Ref.** = secção de [`knudge.md`](knudge.md) que define o comportamento a reproduzir.
> **Aceitação** = comando/observação que decide se a issue está feita (todas verificáveis).

### I-01 — Repositório, toolchain e gate de conformidade
- **Dep.:** —  **Tam.:** S  **Ref.:** §1
- **Objetivo:** monorepo `packages/{core,cli,mcp}` (npm workspaces), TS estrito, os 12 lints do ADR-TS-05, gate `max-lines: 300`, vitest, `pnpm check` = `tsc --noEmit && eslint && vitest run && check-length`.
- **Entregáveis:** `package.json` com `engines.node: ">=22 <23"`, `tsconfig` estrito, `.eslintrc` com as proibições, script de comprimento, `CONTRIBUTING.md` com a tabela de níveis C1–C8 (§2).
- **Aceitação:** `pnpm check` verde com um ficheiro de teste com 301 linhas a **falhar** o gate; `grep -r "Date.now()\|process.env\|Math.random" packages/*/src` devolve vazio.
- **Risco:** falso conforto — os lints não substituem o compilador Rust; documentar o que *não* é coberto (D2).

### I-02 — Oráculo diferencial (`knudge-diff`)
- **Dep.:** I-01  **Tam.:** M  **Ref.:** §6.1, §9
- **Objetivo:** ferramenta que (1) gera um corpus sintético determinístico, (2) corre a mesma sequência em `kd` (Rust) e no port, (3) compara camada a camada (C1–C8) e emite um relatório com o **primeiro byte divergente** e a localização.
- **Entregáveis:** gerador de corpus (reaproveitar o modelo do `bench/src/quality.rs`, §6.3), executor com `Clock` fixo, comparador por camada, relatório Markdown/JSON, corpus golden versionado com **≥200 notas e casos de borda** (acentos NFC/NFD, CRLF, emoji, ligaduras, corpo vazio, 31 chaves, `claims`).
- **Aceitação:** corrido contra o Rust, o relatório diz "0 divergências" para um no-op (o Rust comparado consigo próprio); injectar uma mutação artificial (trocar um byte) e ver o relatório apontar ficheiro/offset.
- **Risco:** é o instrumento de medida de todo o épico — um erro aqui invalida tudo. **Testar o testador.**

### I-03 — Codec TOON (parse + emit) byte-exato
- **Dep.:** I-02  **Tam.:** L  **Ref.:** §2
- **Objetivo:** parser/emissor TOON com as regras do contrato: ordem canónica de 31 chaves, opcionais omitidos (nunca `null`/`undefined`), UTF-8 cru sem BOM, LF final, documento vazio = 0 bytes, escapes `\n\r\t\0\u{}`, aninhamento ≤32, escalares Unicode, inteiros nunca com `.0`, tolerância na leitura (CRLF, espaços) e canonicidade na emissão.
- **Entregáveis:** `toon.ts` (parse/emit/split_frontmatter), tabela de chaves canónica como **array ordenado** (ADR-TS-06), erros tipados com posição (linha/coluna/offset), fixtures golden copiadas de `crates/knudge-cli/tests/golden/`.
- **Aceitação:** round-trip byte-exato sobre todos os goldens; fast-check gerando TOON válido → `emit(parse(x)) === x`; rejeição estrita dos inválidos (chave desconhecida no write, `null`, comentário, tab na indentação).
- **Risco:** D5/D9 — o emissor nunca pode depender da ordem de chaves de um objeto JS (usar a lista canónica).

### I-04 — Schema, nota, `id` e hashes
- **Dep.:** I-03  **Tam.:** L  **Ref.:** §2
- **Objetivo:** `NoteType` (10), `Scope`, `Classification`, `Status`, `EdgeKind` (12), `Value` fechado; `note_id = <tipo>_<base36(8)>` de `hash(type + U+001F + normalize(statement))`; `body_hash = hex8(normalize(statement) + LF + normalize(body))`; `base36_8`/`hex8`; `created_at` UTC ms com sufixo `Z`; `SCHEMA_VERSION=2` com `claims`/`provenance`/ontologia.
- **Entregáveis:** `schema/`, `id.ts`, `hash.ts` (SHA-256 via `node:crypto`, 4 primeiros bytes → hex8), validação de `REQUIRED_KEYS` (5), layout de caminho `notas/<tipo>/<id>.md`.
- **Aceitação:** `note_id` e `body_hash` **idênticos** aos do Rust para o corpus golden completo; `Date` formatado como `2026-01-02T03:04:05.678Z` (comparar com `toISOString` e com o Rust, incluindo ms=0 e ms=999); reclassificar `type` **não** muda o `id`.
- **Risco:** confundir NFC (identidade) com NFD (busca) — o teste tem de cobrir `café` composto vs decomposto a gerar o **mesmo** id.

### I-05 — `formatF64`, União e normalização Unicode
- **Dep.:** I-04  **Tam.:** M  **Ref.:** §2, §3b deste documento
- **Objetivo:** emissor numérico próprio que reproduza `{}` do Rust (decimal, nunca expoente), trate `-0` → `-0`, `NaN`/`Infinity` como caso proibido, e a normalização Unicode (NFC na identidade, NFD+fold na tokenização).
- **Entregáveis:** `format.ts` com teste diferencial contra a lista do Anexo; `unicode.ts` (NFC/NFD, fold de diacríticos, ligaduras, Hangul); golden de 200 casos de acentos/CRLF/emoji.
- **Aceitação:** as 4 divergências medidas (`1e-7`, `1e-10`, `1e+21`, `-0.0`) têm teste dedicado a passar; `formatF64` sobre 10 000 valores aleatórios (fast-check) coincide com o Rust; golden Unicode passa em Node **e** Bun.
- **Risco:** D3/D4 — guardar a versão de ICU/V8 no relatório do oráculo, para que uma atualização de runtime que quebre o golden seja diagnosticável.

### I-06 — Portas, adaptadores Node e fakes
- **Dep.:** I-05  **Tam.:** M  **Ref.:** §3
- **Objetivo:** as 8 portas (`Clock`, `Rng`, `Env`, `Fs`, `Git`, `HookRunner`, `Logger`, `Embedder`) como interfaces TS, com adaptadores `node:*` e **8 fakes** equivalentes (`FixedClock`, `SeqRng`, `FakeEnv`, `MemFs`, `FakeGit`, `NoopHookRunner`, `RecordingLogger`, `FakeEmbedder`).
- **Entregáveis:** `ports/` + `adapters/node/` + `fakes/`; `MemFs` com árvore em memória e tempos injetáveis.
- **Aceitação:** toda a suíte do núcleo corre com fakes e **sem tocar o disco** (`test` falha se um teste escrever em `os.tmpdir()`); lint impede `Date.now`/`process.env` fora do adaptador.
- **Risco:** D7 — garantir que nenhuma porta do núcleo devolve `Promise` (ADR-TS-03).

### I-07 — Store: caminhos, locks e escrita atómica
- **Dep.:** I-06  **Tam.:** L  **Ref.:** §3, Anexo (§3d)
- **Objetivo:** árvore `.knudge/` (`notas/`, `eventos/`, `.idx/`, `.locks/`), escrita **tmp + rename** no mesmo diretório com `fsync` em lote, **lock por ficheiro-alvo** compatível com o Rust: `.locks/<alvo>.lock` criado com `wx`, conteúdo `{"at": ms}`, `stale 30 000 ms`, `retries 50`, `jitter 20 ms` (via porta `Rng`), reclaim por rename para `.stale`, ordem externo-antes-de-interno, libertação por `try/finally` + remoção do ficheiro.
- **Entregáveis:** `store/`, `lock.ts`, `atomic.ts`, `sweep.ts` (locks órfãos), testes de concorrência com processos reais.
- **Aceitação:** **teste de interoperabilidade**: processo Rust (`kd write`) e processo TS a escrever concorrentemente no mesmo repo → sem perda, sem nota truncada, e cada um respeita o lock do outro; `kill -9` a meio da escrita deixa `.idx.new/` + `notas/` consistente.
- **Risco:** é o ponto onde um erro destrói dados do utilizador — a issue tem de ter teste com `SIGKILL`.

### I-08 — Eventos JSONL, `doctor` base e integridade
- **Dep.:** I-07  **Tam.:** M  **Ref.:** §3, §5
- **Objetivo:** `eventos/events.jsonl` (append, rotação, `merge=union`), **ordem de commit nota-antes-do-evento**, e `doctor` base (semântica de saúde, `--audit`, `--fix` apenas do que é seguro).
- **Entregáveis:** `events.ts` (encode/decode/append com fsync), `health/doctor` com o mesmo conjunto de verificações e a mesma ordem de saída, exit codes de `doctor` iguais.
- **Aceitação:** C5 byte-a-byte para uma sequência de 200 operações; simular crash entre nota e evento → `doctor` reporta exatamente o mesmo que o Rust.
- **Risco:** a ordem de commit é contraintuitiva; qualquer inversão produz divergência silenciosa em auditoria.

### I-09 — `init`, config, git e artefactos de projeto
- **Dep.:** I-08  **Tam.:** M  **Ref.:** §3, §5
- **Objetivo:** `kd init` byte-a-byte igual: `config.toml` (provider `http`, mode `lazy`, limiares de recall, `rrf_k`, pesos), `AGENTS.md`, `.gitattributes`/`exclude` (`.idx/`, `.locks/`), resolução de projeto a partir do `git` (worktree principal).
- **Entregáveis:** `config` (leitura tolerante com defaults, escrita canónica), `git.ts` (invocação de `git` externo, sem lib), `init.ts`.
- **Aceitação:** `diff -r` entre `.knudge/` criado pelo Rust e pelo TS = vazio (exceto `.idx/`, por C6); `kd config get/set` com envelope igual.
- **Risco:** TOML tem bordas (ordem, floats) — usar emissor próprio, nunca uma lib (ADR-TS-02).

### I-10 — Tokenização, *stemming* PT e índice invertido
- **Dep.:** I-09  **Tam.:** L  **Ref.:** §4.1
- **Objetivo:** pipeline de pré-processamento (NFD + fold → stopwords PT+EN → stemmer PT conservador com radical ≥4 bytes e plural antes do corte derivacional) e índice invertido com `INDEX_FORMAT = "retrieval-v4-ts"` (ADR-TS-07), `df` por campo e `avg_len` por campo.
- **Entregáveis:** `retrieval/tokenize.ts`, `stem.ts` (regras PT do D206), `index/build.ts` (multi-thread opcional só se medido), serialização do índice em JSONL canónico.
- **Aceitação:** `terms()` idêntico ao Rust para o corpus golden (o Rust expõe os termos via `--json` de diagnóstico, ou comparar `df` por termo); `decoder/decoders`, `configuração/configurações` convergem.
- **Risco:** é onde o ganho do `t11_stemming.md` (+25 % nDCG@5) foi ganho — divergir aqui sabota toda a busca.

### I-11 — BM25 por campo (bit-exato)
- **Dep.:** I-10  **Tam.:** M  **Ref.:** §4.1
- **Objetivo:** `score(d,q)` com `k1=1.5`, `b=0.75`, pesos de campo `3/2/1`, `IDF` de Robertson-Sparck-Jones por campo, `type_weight` (8 valores), boost `1 + 0.1·confirmação`, corte de alta frequência (`df/N ≥ 0.9` só com `N ≥ 64`), `df` = máximo entre campos.
- **Entregáveis:** `retrieval/bm25.ts`, `type_weight.ts`, teste de bits.
- **Aceitação:** para 100 consultas × 200 notas, o **conjunto de scores ordenados** coincide bit a bit com o Rust (comparação de `BigUint64` dos `f64`); o limiar de corte coincide em `N=63/64`.
- **Risco:** `Math.log1p` mediu 0 divergências (§3b), mas somar em ordem diferente muda o resultado — a ordem de soma é contrato.

### I-12 — Canais, RRF, rank e `ask`
- **Dep.:** I-11  **Tam.:** L  **Ref.:** §4.2, §5
- **Objetivo:** canais lexical/âncora/vetorial/PPR, `RRF(x) = Σ peso_c/(k + r_c(x) + 1)` com `k=60` e pesos `1/2/30/0`, penalidade de contradição, confiança/idade/drift, desempate `(score desc, id asc)`, snippet/`why`, modo `strict`, envelope de `ask`.
- **Entregáveis:** `retrieval/{anchors,rrf,rank}.ts`, `commands/ask.ts`.
- **Aceitação:** **K2**: os top-10 de 60 consultas do corpus de qualidade são idênticos (mesmos ids, mesma ordem) aos do Rust; envelope `--json` byte-a-byte.
- **Risco:** a soma do RRF é sensível à ordem de iteração dos canais (ADR-TS-06) e o `ppr=0` desligado tem de continuar a ser *exatamente* 0 (não entrar na soma).

### I-13 — Ciclo de vida, classificação e dedup por limiares
- **Dep.:** I-12  **Tam.:** L  **Ref.:** §4.3, §4.5, §5
- **Objetivo:** shelf-life por `classification`, `lifecycle` (FSRS-like + confiança), dedup de duas fases com limiares `<0.75` cria / `0.75–0.92` funde / `≥0.92` rejeita, e a peneira exata de postings.
- **Entregáveis:** `write/dedup/{sieve,mod}.ts`, `write/lifecycle.ts`, `write/draft.ts`.
- **Aceitação:** para o corpus golden, o **score de dedup** coincide bit a bit e a **decisão** (criar/fundir/rejeitar) é a mesma em 100 % dos pares candidatos; `write` mantém C1.
- **Risco:** o score é a soma de componentes ordenadas — replicar a ordem exata de composição.

### I-14 — MinHash/LSH em `BigInt` e propostas de merge
- **Dep.:** I-13  **Tam.:** L  **Ref.:** §4.3, Anexo (§3d)
- **Objetivo:** FNV-1a de 64 bits + splitmix64 (64 permutações) + *banding* 16×4, ativados só com `N > 256` e vocabulário denso; `u64::MAX` em conjunto vazio; pares candidatos em ordem canónica; validação final pelo Dice exato.
- **Entregáveis:** `write/dedup/lsh.ts` com `BigInt.asUintN(64, …)`, teste de vetores conhecidos (assinaturas e bandas iguais às do Rust), medição de custo.
- **Aceitação:** **C7/K3**: assinaturas iguais às do Rust para o corpus de teste; propostas idênticas ou superconjunto em ≥99 % dos casos; `doctor` denso (1000 notas) abaixo de 1 s.
- **Risco:** D6 — se `BigInt` tornar o `doctor` denso lento, implementar em `Uint32Array` de dupla palavra **mantendo** os mesmos bits.

### I-15 — `write`, `update`, `forget`, `batch`, `--dry-run`
- **Dep.:** I-14  **Tam.:** L  **Ref.:** §5
- **Objetivo:** os verbos de mutação com atomia, locks, batch JSONL (teto 100), `--dry-run`, ordem de commit, propagation de `forget` (TMS: `retracted`/`defeated_dependents`, sem apagar).
- **Entregáveis:** `commands/{write,update,forget}.ts`, `write/batch.ts`, `write/reconcile.ts`.
- **Aceitação:** C1 byte-a-byte para 300 operações mistas (incluindo conflitos, duplicados, `--dry-run` que não escreve nada); `update` reescreve um `id` estável.
- **Risco:** recursos a meio de um batch (ficheiro ilegível) têm de degradar como o Rust (`skip` com warning + orientação), não a abortar o batch.

### I-16 — Grafo, ontologia, TMS e PPR/Louvain
- **Dep.:** I-15  **Tam.:** L  **Ref.:** §4.4
- **Objetivo:** `EdgeKind` (12) + ontologia (`same_as`/`broader`/`narrower`/`related` com inversos), `claim_conflicts`, propagação `defeasible`, PPR (damping 0.85, ≤32 iterações, L1 `1e-8`, ordem canónica de ids), Louvain determinístico (≤8 níveis, empate mantém comunidade), sugestões em `.idx/suggestions.jsonl`, verbo `map`.
- **Entregáveis:** `graph/{edges,ontology,tms,ppr,louvain}.ts`, `commands/map.ts`.
- **Aceitação:** scores de PPR bit-idênticos ao Rust no corpus golden (a ordem canónica dos ids é o que garante isto) e a mesma partição Louvain; `claim_conflicts` idênticos.
- **Risco:** ponto flutuante em iteração: qualquer acumulação em ordem diferente altera o resultado — replicar a ordem de soma do Rust, item a item.

### I-17 — Tarefas, fluxo, confiança e manutenção
- **Dep.:** I-16  **Tam.:** L  **Ref.:** §4.5, §5
- **Objetivo:** `task` (estados, dependências, `ready`, épicos por `scope=epic`), `outcomes` → confiança **Beta-Bernoulli + Wilson (z=1.96)** + feedback/tarefa/recência + `drift_factor`, drift de âncoras e de termos (JS-divergence ≥0.5 com ≥4 notas), `maintenance prune/degrade`, `compact`, `sync`.
- **Entregáveis:** `task/flow.ts`, `lifecycle/confidence.ts`, `maintenance/*.ts`.
- **Aceitação:** confiança bit-idêntica (o Wilson mediu 0 divergências, §3b); `task list --ready` com a mesma ordem; `maintenance prune` propõe exatamente o mesmo conjunto.
- **Risco:** a ordem dos desempates (`impacto desc, created asc, id asc`) é contrato — sem ela, a saída muda entre runs.

### I-18 — Embeddings e auto-drain
- **Dep.:** I-17  **Tam.:** M  **Ref.:** §4.6
- **Objetivo:** provedores `none`/`lightweight`(hash)/`http` (endpoint OpenAI-compatible), cache `.idx/embeddings.jsonl` com metadados de modelo/dimensões, invalidação por mudança de modelo, `drain` e auto-drain `lazy` por lote após cada verbo (com exclusão em `doctor`/`drain`/`maintenance`), canal semântico na fusão.
- **Entregáveis:** `embeddings/{provider,cache}.ts` (único ponto `async`, ADR-TS-03), `commands/drain.ts`.
- **Aceitação:** em modo `lightweight`, os mesmos ids/ordens no canal vetorial que o Rust; auto-drain respeita o *kill switch* de ambiente e não reintroduz os 40 ms de custo oculto (§6.2) nos verbos triviais; **K4** medido.
- **Risco:** `fetch` com servidor ausente tem de degradar com warning e nunca bloquear a CLI.

### I-19 — CLI completa, `prime`, `rewind` e contrato de máquina
- **Dep.:** I-18  **Tam.:** L  **Ref.:** §5
- **Objetivo:** os 14 verbos, `stdout`=dados/`stderr`=logs, envelope `{success, command, data?, error{code,message,retryable}, warnings?}`, exit codes por `ErrorKind` (2–9, 70), `EPIPE` → 0, `prime` (golden byte-exato), `rewind` com orçamento `ceil(len/4)`/`DEFAULT_BUDGET=4000`, `context_id` + `--resume`, `self version/completions`, `config`.
- **Entregáveis:** `cli.ts` (parser próprio ou `node:util.parseArgs` — evita dependência), `commands/*.ts`, tabela de exit codes como dado.
- **Aceitação:** **C2 e C3** a 100 % contra 8 goldens + 40 casos de erro; `rewind --resume` byte-idêntico com `Clock` fixo; `kd help` em PT (a divergência de idioma do `clap` é uma melhoria catalogada, §8.5 de `knudge.md`, não parte do contrato).
- **Risco:** este é o entregável visível; a tentação de "melhorar" mensagens quebra C2 — qualquer melhoria entra depois, com `DIVERGENCES.md` assinado.

### I-20 — MCP: 4 gatilhos sobre stdio
- **Dep.:** I-19  **Tam.:** M  **Ref.:** §5
- **Objetivo:** JSON-RPC 2.0 sobre stdio com `knudge_pre_write`, `knudge_pre_edit`, `knudge_session_end`, `knudge_status`; *hints* como **ponteiros** (`id + statement + score`), nunca corpos; mesmos formatos de resposta que o Rust; modo degradado sem projeto.
- **Entregáveis:** `packages/mcp` (zero deps; JSON-RPC escrito à mão ou `@modelcontextprotocol/sdk` como *devDep* de teste), geração das configs dos clientes.
- **Aceitação:** transcrição de sessão gravada (request/response por linha) idêntica byte-a-byte à do Rust para a mesma sequência de gatilhos; teste de "nunca devolver corpo de nota".
- **Risco:** sem projeto, o Rust hoje pode devolver vazio silenciosamente (§8.4 de `knudge.md`) — reproduzir o comportamento **e** registar a borda, para não "corrigir" o contrato por engano.

### I-21 — Empacotamento, distribuição e CI
- **Dep.:** I-20  **Tam.:** M  **Ref.:** §1, §9
- **Objetivo:** bundle único (`esbuild`) + binário nativo por plataforma (`bun build --compile` **ou** Node SEA), pacote npm com `engines` fixados, `install.sh` equivalente (verificação SHA-256, instalação não-destrutiva), CI em matriz (Linux/macOS/Windows) com `pnpm check`, auditoria de deps e **teste de versão** (goldens C1/C3).
- **Entregáveis:** `esbuild` config, workflow CI, script de release, `CHANGELOG` do porte.
- **Aceitação:** `kd` instalado do artefacto distribuído passa os goldens em Node **e** em Bun na versão fixada; piso de arranque do artefacto ≤15 ms (**K4**); `npm pack` sem runtime deps.
- **Risco:** D8/D10 — qualquer dep transitiva nova tem de passar `deny`-equivalente.

### I-22 — Auditoria final de fidelidade, *bench* próprio e coexistência
- **Dep.:** I-21  **Tam.:** M  **Ref.:** §6, §8
- **Objetivo:** relatório final do oráculo (C1–C8, taxa de conformidade por camada), `DIVERGENCES.md` do port (com as 4 bordas de formatação, o namespace do `.idx` e o LSH aproximado), bancada própria (micro/e2e/**qualidade** com o mesmo dataset sintético e as mesmas métricas Recall@k/MRR/nDCG@k) e guia de coexistência Rust↔TS no mesmo repositório.
- **Entregáveis:** `relatorio-fidelidade.md`, `DIVERGENCES.md`, `bench/` em TS (medianas, min/p95), guia de contorno de *kill criteria*.
- **Aceitação:** nº de divergências **explicadas** = nº de divergências observadas (nenhuma órfã); as métricas de qualidade do port iguais às do Rust (top-k idênticos ⇒ métricas iguais, o que valida C6); custo por nota a 1000 e 10 000 notas documentado.
- **Risco:** tentação de "fechar" o épico sem reproduzir o *bench* — sem ele não há como negociar desempenho depois.

---

## 10. Riscos residuais (aceites, fora do âmbito do épico)

1. **Não é uma migração de dados** — é um segundo motor sobre o mesmo formato. Quem quiser "só TS"
   herda o risco D11 (duas implementações) até o oráculo Rust sair do CI.
2. **Extensões `claims`/`provenance` (§2)** ficam sem uso avançado até haver consumidores; o porte
   entrega-as como dados, não como funcionalidade nova.
3. **Nada neste plano melhora a qualidade de retrieval** (`knudge.md` §8.2: corpus sintético, n=12
   por família, tetos saturados). Se o objetivo científico é medir melhor, isso é um épico separado
   (corpus rotulado real + baselines externos) — e é mais decisivo que a linguagem.
4. **A regra "sem daemon" continua a ser reafirmada** (ADR-TS-01): se o piso de arranque vier a ser
   intolerável, a saída é MCP-only (long-lived por natureza), não um daemon.

---

## Anexo A — comandos que produziram as medições (§3)

As fontes das sondas estão em [`medicoes/`](medicoes/) (ficheiros pequenos, sem dependências):
`num2.rs` + `num2.mjs` (fidelidade numérica e formatação), `path.rs` + `path.mjs` (data path),
`gen.mjs` (gera o corpus), `cmp.sh` (comparador textual), `med.sh`/`med2.sh` (medianas). Correr
num diretório de trabalho vazio — `gen.mjs` escreve `./bench-corpus/`.

```bash
# (a) piso de arranque: mediana de 15 execucoes por comando
kd --help ; kd self version
node -e "" ; bun -e "" ; deno eval "" --no-config
node -e "require('node:fs');require('node:crypto');require('node:path')"

# (b) fidelidade numerica: 51 sondas (log1p/log/sqrt/exp/pow + formatacao numero->texto)
rustc -O num2.rs -o num2_rs && ./num2_rs > out_rs.txt && node num2.mjs > out_node.txt
bash cmp.sh out_rs.txt out_node.txt          # 51 probes / 47 iguais / 4 divergentes (todas 'fmt')

# (c) data path: 200 ou 1000 notas, mediana de 11 execucoes
node gen.mjs 1000 ; rustc -O path.rs -o path_rs ; ./path_rs ; node path.mjs ; bun path.mjs
```

> **Armadilha metodológica que apanhámos** (e que fica documentada porque é fácil de repetir): a
> comparação direta com `awk '$1==$2'` compara **números**, não texto — e `1e+21` e
> `1000000000000000000000` são numericamente iguais, o que **mascarou** as 4 divergências na
> primeira passagem. `cmp.sh` força contexto textual com uma sentinela. A segunda armadilha foi
> comparar `ln(1+x)` (Rust) com `ln(2+x)` por erro na sonda: os 6 "desvios" de `idf` da primeira
> tentativa eram **meu** erro de fórmula, não do V8. Sempre que uma sonda acusar divergência
> numérica, a primeira hipótese é a sonda.

Ambiente: AMD Ryzen 5 5500U (12 threads), Linux 7.2.10, `kd` 0.6.1, rustc 1.99.0, Node 22.23.2,
npm 10.9.8, Bun 1.4.2, Deno 2.9.5. Máquina de desenvolvimento **não isolada**: os valores ordenam
custos, não fixam números de produção (mesma reserva do `bench/RELATORIO.md` do repo).

## Anexo B — entradas do oráculo que **têm** de existir (casos de borda)

| Categoria | Casos mínimos |
|---|---|
| TOON | documento vazio (0 bytes), LF final ausente, CRLF, tab em indentação, chave desconhecida, `null` explícito, aninhamento 32/33, escalar Unicode, escapes `\u{}`, lista inline, lista de mapas |
| Unicode | `café` NFC/NFD, ligadura `ﬁ`, Hangul composto/decomposto, emoji ZWJ, BOM |
| Números | `0.7`, `1.0`, `-0.0`, `1e-7`, `1e-10`, `1e20`, `1e21`, `0.1+0.2`, `NaN`/`Infinity` (devem ser rejeitados) |
| Tempo | `ms=0`, `ms=999`, `created_at` com Z, relógio a recuar, `now_ms` fixo |
| Ids | reclassificar `type`, `statement` só com espaços, `statement` com `U+001F`, colisão forçada |
| Ficheiros | nota truncada a meio, nota com 0 bytes, `.idx` corrompido, lock *stale*, lock de outro PID, symlink |
| Corpus | 63 e 64 notas (limiar do corte de alta frequência), 256 e 257 (limiar do LSH), 1000 densas e esparsas |
