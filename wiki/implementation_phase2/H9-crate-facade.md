# H9 — Crate facade

> **Issue:** `task_00dkc6dk` · **Estado:** **concluída** · **Âncora:**
> [crate-facade](../backlog/crate-facade.md) · **Desenho:**
> [revisao-proposito](../revisao-proposito.md) §2 (uso #1) · **Pré-requisitos:** H5 ·
> **Contrato:** [fachada-crate](../fachada-crate.md).

## Objetivo
Uma **fachada** pública e estável para embutir o ktu noutro projecto Rust (uso #1), escondendo o
kernel atrás de `Ktu`/`KtuBuilder` — como `Knudge`/`KnudgeBuilder` (D214).

## Passos
1. **`Ktu`/`KtuBuilder`** — `root`, `ports` (Clock/Rng/Fs/Env/Logger/Metrics), `provider`; `build()`
   resolve o projecto e monta o kernel.
2. **`turn`/`session`** + tipos públicos estáveis (`TurnOutcome`, `Event`); o embedder **nunca** vê
   `Deps`/`LoopConfig`/`run_turn`.
3. **Exemplo + doc + teste** de embedding (um crate de exemplo que abre uma sessão e corre turnos).

## Tarefas kd
| Tarefa | Entrega |
|---|---|
| `task_01jalrhj` | `Ktu`/`KtuBuilder` |
| `task_00ghiyx7` | `turn`/`session` + tipos públicos |
| `task_01nj66mb` | Exemplo + doc + teste de integração |

## Gate / testes
- O exemplo compila e corre um turno com portas fake (determinístico).
- `ktu-core` mantém-se puro; a fachada **esconde** o kernel (o embedder só vê portas/eventos).
- `xtask check-layers` verde (a fachada não quebra o firewall).

## Riscos
- Expor demais → esconder o kernel; `run_turn`/`Deps` deixam de ser API pública a longo prazo.
- API instável → a fachada é o **único** caminho suportado de integração.

## Estado da implementação

`Ktu`/`KtuBuilder`/`TurnOutcome` em `crates/ktu-core/src/facade/` (módulo `mod.rs` + `builder.rs`,
ambos ≤ 400 linhas), com o **exemplo de incorporação como doc test executável** e o teste de
integração `crates/ktu-core/tests/embed.rs`. O `ktu-cli` **deixou de ter composição própria**:
`turn_runtime::execute_turn` monta os adaptadores reais e delega na fachada.

**Documentação de integração:** `wiki/integration/` (índice + 10 tópicos: panorama, construtor,
turnos, sessões, portas/adaptadores, projecto, erros/diagnóstico, boas práticas, exemplos e um
contrato resumido para agentes de IA) e o **crate de exemplo compilado**
`examples/ktu-embed` (membro do workspace, no gate `make check`).

**Decisão de desenho (documentada):** a fachada vive no `ktu-core` e recebe **tudo por injecção**
(portas + efeitos). O `ktu-core` é puro (M2) e não constrói adaptadores `std`; a alternativa
(crate nova com adaptadores, como o `Knudge`/`D214` do knudge) fica adiada até haver um embedder
que não queira montar as suas portas. Consequência: a composição **pura** (guardrails,
permissões, system prompt, orçamento, modo→`AgentSpec`, sessão) fica num só lugar e é partilhada
pela CLI e por quem embute (M4).

**Desvio ao plano:** o construtor não recebe `Env`/`Rng` (portas declaradas **sem consumidor** no
loop — seria cerimónia) e recebe `budget`/`cancel`/`registry`/`plan_spec` (controlo e escolha do
agente), que o plano não enumerava mas que a composição exige.

## Notas de execução (skills Rust)
> Ver [NOTAS-RUST](./NOTAS-RUST.md) (transversais).
- **`16` Builder:** seguir o padrão da skill, mas com **enums** em vez de flags (N8).
- **N2:** o builder nunca usa `unwrap`; `build()` devolve `Result`.
- **Doc tests (`13`):** o exemplo de embedding é um **doc test** executável.
- **M2:** a fachada **esconde** o kernel — o embedder só vê portas/eventos; `run_turn` deixa de ser
  API pública.
- **Gate:** **G12** (o caminho crate deve dar o mesmo log).

## Prova (G12 — independência do caminho)

```
$ cargo test -p ktu-core --doc        # o exemplo de incorporação corre (não é só compilado)
test crates/ktu-core/src/facade/mod.rs - facade (line 15) ... ok

$ cargo test -p ktu-core --test embed # incorporação com fakes, de fora do crate
test an_embedder_runs_a_tool_turn_and_persists_the_session ... ok
test the_same_script_gives_byte_identical_logs ... ok
test the_injected_cancel_token_stops_the_turn ... ok

$ cargo test -p ktu-cli --test session
test run_and_repl_produce_the_same_log ... ok     # M4: run e REPL, o mesmo log
```

| Critério | Prova |
|---|---|
| O exemplo compila **e corre** com portas fake | doc test executável em `facade/mod.rs` (sem `no_run`) |
| A fachada esconde o kernel | `tests/embed.rs` só usa `Ktu`/`TurnOutcome`/`SessionLog`/`Event` + portas |
| `ktu-core` mantém-se puro | `xtask check-layers` verde (nenhuma dependência nova) |
| Modo plano read-only estrutural (I15) | `plan_mode_offers_only_the_read_only_tools`; sem `plan_spec` ⇒ recusa |
| Fail-closed nos guardrails/permissões (I16) | `a_broken_guardrail_stops_the_build`, `broken_permissions_stop_the_build` |
| O caminho crate dá o **mesmo** log | a CLI e a fachada partilham a composição (`execute_turn` → `Ktu`); o determinismo é travado por `the_same_script_gives_byte_identical_logs` e por `run_and_repl_produce_the_same_log` (M4) |
| Gate | `make check` verde (ktu) |

> A prova **byte-a-byte** dos três caminhos (`run`/REPL/crate) na mesma sessão é formalizada em
> **H12** (G10–G15); aqui fica a prova de que a composição é única e determinística.
