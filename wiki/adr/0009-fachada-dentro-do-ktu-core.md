# 0009 — A fachada de incorporação vive no `ktu-core`

- **Status:** aceite
- **Supersedes:** —
- **Superseded by:** —
- **Âncora:** `crates/ktu-core/src/facade/`, `examples/ktu-embed/`, `wiki/fachada-crate.md`

## Contexto

O ktu é um binário, mas o valor do kernel é incorporável: outro programa Rust quer correr turnos
sem reimplementar a composição (provider + tools + guard + orçamento + memória + subagentes). A
CLI já fazia essa composição à mão; duplicá-la em cada consumidor seria uma segunda fonte de
verdade (M4).

## Decisão

A fachada `facade::{Ktu, KtuBuilder, TurnOutcome}` vive **dentro do `ktu-core`** e recebe **tudo
injectado** (portas e adaptadores do chamador). A CLI passa a delegar nela, e o exemplo
`examples/ktu-embed` (membro do workspace) prova que incorpora com ~30 linhas de adaptadores.

## Alternatives considered

| Alternativa | Porque não |
|---|---|
| Crate nova (`ktu-sdk`) sobre o core | mais uma camada a versionar; a fachada não tem dependências próprias |
| Expor `Deps`/`run_turn` directamente | obriga cada consumidor a compor a cadeia de guardrails e o orçamento — foi o que causou duplicação |
| Fachada a construir os adaptadores `std` (FS/relógio/log) por ti | mistura domínio com borda ([0001](0001-kernel-puro-com-portas.md)); fica adiada como conveniência |
| `Env`/`Rng` no construtor | portas sem consumidor no loop (YAGNI) |

## Consequências

- Um só caminho de composição: CLI, TUI, servidor HTTP, subagentes e consumidores externos.
- O consumidor continua dono dos adaptadores e do log (o `SessionLog` é explícito), pelo que I3 se
  mantém verificável de fora.
- Custo: a fachada tem de acompanhar o `Deps` interno; os testes de `facade/tests.rs` e o exemplo
  no gate mantêm as duas pontas alinhadas.
