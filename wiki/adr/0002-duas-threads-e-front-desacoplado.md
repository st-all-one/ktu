# 0002 — Duas threads (kernel + tools) e front desacoplado

- **Status:** aceite
- **Supersedes:** —
- **Superseded by:** —
- **Âncora:** `crates/ktu-core/src/agent/`, `crates/ktu-tools/src/tool_thread.rs`,
  `crates/ktu-port/src/`

## Contexto

Um agente precisa de correr tools lentas (bash, HTTP) sem bloquear o loop, e de servir front-ends
diferentes (CLI, TUI, HTTP) sem duplicar lógica. As tentativas anteriores misturavam as três coisas
num `async` só, com cancelamento ambíguo e estados partilhados.

## Decisão

**T1** é o kernel: loop, sessão, guard, orçamento — um runtime `current_thread`. **T2** é a thread
de tools: dona dos adaptadores lentos, fala com o kernel por **mensagens** (canal), nunca por estado
partilhado. O front-end (CLI, TUI, HTTP) fala com o kernel por uma **porta única**
(`ktu-port::Port`), também por mensagens. Nada partilha estado mutável com o kernel (I8).

## Alternatives considered

| Alternativa | Porque não |
|---|---|
| Um único runtime `async` multitarefa | tools lentas e loop competem pelo mesmo executor; cancelamento e pausa tornam-se difusos |
| `tokio::spawn` por chamada de tool | o `select!` está banido (usa `panic!`), e o ciclo de vida das tools fica sem dono |
| Actor por tool | multiplica fronteiras e não resolve nada que a T2 não resolva |
| Front-end dentro do kernel (TUI na thread do loop) | a TUI passaria a influenciar o timing do turno; o HTTP teria de reimplementar o loop |

## Consequências

- O kernel é `Send` na fronteira e o estado vive só no log ([0003](0003-log-append-only-e-a-fonte-da-verdade.md)).
- A TUI e o servidor HTTP são **clientes**: nenhum caminho de decisão novo (M4).
- Custo: um canal e um protocolo de mensagens para manter; em troca, um único dono do cancelamento
  (I4) e da fronteira de confiança.
