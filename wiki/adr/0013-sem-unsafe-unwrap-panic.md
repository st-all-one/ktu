# 0013 — Sem `unsafe`/`unwrap`/`panic`; `allow` com `reason`

- **Status:** aceite
- **Supersedes:** —
- **Superseded by:** —
- **Âncora:** `clippy.toml`, `Cargo.toml` (`[workspace.lints]`)

## Contexto

O katu é um agente que corre comandos e escreve ficheiros: uma falha não tratada não é um *bug* de
apresentação, é trabalho perdido. E `unsafe` num kernel puro é uma porta que não precisamos de
abrir. Ao mesmo tempo, a lista de lints só tem valor se não for contornada em silêncio.

## Decisão

- `unsafe` é **proibido** (`#![forbid(unsafe_code)]` no workspace e nos bins).
- `unwrap`/`expect`/`panic!`/`todo!`/`unreachable!` estão banidos por lint **em `src/` e em testes**;
  `print_stdout`/`print_stderr`/`exit` também (o `stdout` é dado e o `stderr` é log: só pelos
  módulos `output`/`run::report_error`).
- `#[allow(...)]` exige **`reason`** — a excepção fica documentada no sítio onde é feita.
- Os macros/métodos perigosos (`std::panic`, `std::unreachable`, `SystemTime::now`,
  `std::env::var`) estão em `disallowed_macros`/`disallowed_methods`.

## Alternatives considered

| Alternativa | Porque não |
|---|---|
| Permitir `unwrap` em testes | os testes são código de produção em confiança: um `unwrap` esconde a causa de uma falha |
| `expect("mensagem")` em vez de `unwrap` | mesma categoria: o caminho de erro não é tratado; o lint bane os dois |
| `#[allow]` sem justificação | o lint desaparece do diff e ninguém sabe porquê — foi assim que o katu acumulou excepções |
| `tokio::select!` (usa `panic!` internamente) | banido; o cancelamento é verificado com `try_recv` em fronteiras de passo |
| `unsafe` pontual para performance | não há medição que o peça ([0014](0014-medir-antes-de-optimizar.md)) |

## Consequências

- Todo o caminho de erro é explícito: `Result`/`Option` com erro tipado e remédio.
- Excepções legítimas (borda: relógio, `stdout`, casts de *bench*) existem **com `reason`** — é
  possível lê-las e discuti-las.
- Custo: mais `let Some(x) = … else { … }` e menos uma linha por caminho; em troca, o agente falha
  onde é suposto falhar.
