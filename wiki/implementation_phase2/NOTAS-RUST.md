# Notas de execução — skills Rust × princípios do ktu

> Cruzamento de [`.agents/skills/rust/`](../../.agents/skills/rust/SKILL.md) (Rust **1.99.1** /
> Edition 2024) com os princípios do ktu: **simplicidade · determinismo · Rust 1.99+ · gate de
> qualidade**. Onde a skill **contradiz** o ktu, **o ktu ganha** — a skill é guia, os lints e o
> `make check` são lei. Data: 2026-10-17.

---

## 0. Onde a skill e o ktu estão alinhados (usar como está)

| Tema | Skill | ktu |
|---|---|---|
| Lib + thin binary | `16` §"Biblioteca + Binário" | `ktu-core` + `ktu-cli` |
| Injeção por traits | `16` #5 | portas (`Clock`/`Fs`/…) + fakes |
| Organizar por domínio | `16` #1 | módulos por domínio |
| `proptest` (property) | `13` #9 | já usado em 5 crates |
| Release profile | `16` | `lto=fat`, `codegen-units=1`, `panic=abort` |
| ADRs | `16` #7 | H11 |
| `cargo deny`/audit | `16` #8 | `make supply-chain` |
| `thiserror` em lib | `16` #4 | `Error` tipado próprio (equivalente) |

---

## 1. Onde a skill **contradiz** o ktu (o ktu ganha)

| # | Skill diz | ktu exige | Ação |
|---|---|---|---|
| **N1** | `HashMap<K,V>` é o **map default**; "`BTreeMap` se precisa de ordenação" (`05`) | **`HashMap`/`HashSet` proibidos** — só `BTreeMap`/`BTreeSet` (determinismo) | **Nunca** usar `HashMap`; o lint/`clippy.toml` manda |
| **N2** | exemplos usam `unwrap()` (ex.: `lock().unwrap()`) | **`unwrap`/`expect`/`panic` proibidos** em `src/` **e** testes | Traduzir todo exemplo: `?`, `let else`, `Result` |
| **N3** | `#[should_panic]` para testar erros (`13`) | o ktu **nunca** panica (`panic=abort`); erros são **valores** | Proibido `should_panic`; testar `Err(_)` |
| **N4** | capítulo `unsafe`/FFI/macros (`14`) | `#![forbid(unsafe_code)]` | `14` **não** se aplica a `src/` |
| **N5** | `tracing` + `#[instrument]` (`16`) | `Logger` é **porta** zero-cost; `tracing` só na borda | Não introduzir `tracing` no kernel |
| **N6** | `anyhow` em app | o ktu usa `Error` tipado + mensagem accionável | Manter o `Error` próprio |
| **N7** | `mockall` para mocks (`13`) | **fakes** das portas (determinismo) | Não adoptar `mockall` |
| **N8** | builder com `Option`/flags | `max-fn-params-bools=0`, `max-struct-bools=1` | Flags → **enums** |
| **N9** | `sort_unstable()` "mais rápido" (`05`) | ordem **determinística** e estável quando observável | `sort()` (estável) onde a ordem é contrato |

---

## 2. Rust 1.99+ — o que adoptar e o que **evitar**

| Item | Veredicto |
|---|---|
| `let else`, `gen` blocks, RPIT 2024 | **adoptar** onde simplifica |
| `String::from_utf8_lossy_owned` (1.99) | **adoptar** (evita clone em leitura) |
| `Vec::into_parts`/`from_parts` (1.99) | **adoptar** com cuidado (ownership explícito) |
| `Box::into_non_null` (1.99) | só se necessário (não no kernel) |
| `format_into`/`NumBuffer` (1.98) | **adoptar** em logging/serialização de alto débito |
| `f32/f64::algebraic_*` (1.98) | **EVITAR** — **não-determinístico** (`02`); o ktu faz orçamento/custo com floats reprodutíveis |
| `unsafe extern`/`VaList` (1.99) | **não aplicável** (`forbid(unsafe_code)`) |

> **Regra de ouro do determinismo:** nada que dependa de hash-ordering, tempo de parede, `Instant::now`,
> `env::var` directo ou `algebraic_*` entra no kernel. Tudo por porta (`Clock`/`Rng`/`Env`) ou por
> `BTreeMap`. A skill avisa em `02`; o ktu **proíbe** por lint.

---

## 3. Testes e gate

- **Skill `13`** recomenda `nextest`/`tarpaulin`; o ktu usa `cargo test` + `proptest` + fakes +
  `benches/`. **Não** introduzir `nextest`/coverage sem número que o justifique (M1).
- **Teste = prova da invariante.** Cada tarefa da Fase 2 aponta o **teste** que a prova; nada fecha
  sem ele.
- **Property-based** (`proptest`) é o padrão para parsers/normalizadores (line-endings, tool-call,
  dedup de memória) — usar sempre que a entrada é "qualquer bytes".
- **Golden/byte-idêntico** para o que é contrato (log, `prime`, `.gitattributes`, `AGENTS.md`).

---

## 4. Notas transversais (aplicam-se a **toda** a Fase 2)

1. **Determinismo primeiro:** `BTreeMap`/`BTreeSet`; ordem estável; nada de `HashMap` (N1).
2. **Sem `unwrap`/`panic`** em `src/` nem testes (N2/N3).
3. **Flags → enums** (N8); `#[allow]` sempre com `reason`.
4. **Efeitos só por portas**; `tracing` só na borda (N5).
5. **Ficheiros ≤ 300 linhas** (`check_file_length`); testes/benches isentos.
6. **`make check` + `make supply-chain`** verdes antes de fechar qualquer issue.
7. **1.99 onde simplifica, nunca onde quebra reprodutibilidade** (§2).
