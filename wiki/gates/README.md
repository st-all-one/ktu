# Gates das Fases 2 e 3 — G10–G22 (evidência arquivada)

> **Gates de saída.** Fase 2: `G10`–`G15` verdes. Fase 3: `G16`–`G20` e `G22` verdes (os itens
> sem gatilho não têm gate — ficam no [dossiê](../pendencias.md) com a condição declarada). Em ambos
> os casos, com artefactos arquivados, + `make check` e `make supply-chain` verdes. Os critérios
> de aceitação estão em [`../implementation_phase2/GATES.md`](../implementation_phase2/GATES.md)
> e [`../implementation_phase3/GATES.md`](../implementation_phase3/GATES.md); este directório
> guarda **a prova** (o quadro, os relatórios por gate e os artefactos crus).

## 1. Como correr

```bash
make gates                      # todos os gates (determinísticos) → wiki/gates/{summary.json,artifacts/}
make gates ARGS='--only G12,G13'  # um subconjunto
cargo run -q -p xtask -- gates --real-model --base-url http://127.0.0.1:8080 --model qwen2.5-coder-1.5b
                                # acrescenta a observação com um modelo real (G11)
```

O runner é `xtask gates` (Rust, sem dependências novas): prepara um espaço de trabalho isolado por
gate em `.ktu/gates/<gate>/`, corre o **binário** `ktu` de ponta a ponta, verifica os critérios e
escreve os artefactos em `artifacts/<gate>/` + `summary.json` (contrato de máquina).

**Não** está no `make check` (escreve evidência versionada); corre na CI como passo próprio
(`.github/workflows/ci.yml`). Corre no fecho de fase.

## 2. Quadro (corrida de 2026-10-18)

| Gate | Foco | Veredicto | Verificações | Relatório |
|---|---|---|---|---|
| **G10** | cadeia longa (≥15 turnos, mesma sessão) | ✅ | 7/7 | [G10](G10-cadeia-longa.md) |
| **G11** | robustez multi-modelo (tool call em texto) | ✅ | 2/2 + observação | [G11](G11-multi-modelo.md) |
| **G12** | independência do caminho (M4) | ✅ | 4/4 | [G12](G12-tres-caminhos.md) |
| **G13** | pipeline concorrente | ✅ | 6/6 | [G13](G13-concorrencia.md) |
| **G14** | project-bound (init/config/M3/M5/doctor) | ✅ | 16/16 | [G14](G14-project-bound.md) |
| **G15** | eixo de memória (knudge) | ✅ | 9/9 | [G15](G15-memoria.md) |
| **G16** | *steering* durável e idempotente (M7) | ✅ | 9/9 | [G16](G16-steering.md) |
| **G17** | REPL interactivo (comandos `/` e Ctrl-C) | ✅ | 10/10 | [G17](G17-repl.md) |
| **G18** | recursos globais e piso visível (M8) | ✅ | 10/10 | [G18](G18-recursos.md) |
| **G19** | edição sob concorrência | ✅ | 8/8 | [G19](G19-edicao.md) |
| **G20** | migrações de config | ✅ | 7/7 | [G20](G20-migracao.md) |
| **G22** | travões (links, doc e knudge) | ✅ | 6/6 | [G22](G22-travoes.md) |

`summary.json` é a versão máquina-legível deste quadro (ids, veredictos, verificações, notas).

## 3. Método (o que torna a prova honesta)

1. **Determinismo primeiro.** Os gates correm com `--provider mock` (eco) ou `--provider scripted`
   (guião JSONL) — sem inferência. A corrida completa, duas vezes, dá o **mesmo** quadro (mesmas
   verificações, mesmo veredicto). `G12` prova-o dentro do próprio gate (o mesmo caminho duas vezes
   ⇒ o mesmo log) e a cadeia do `G10` é um guião versionado.
2. **End-to-end pelo binário.** Nada é provado chamando a biblioteca: o gate corre `ktu` e lê o que
   fica em disco (log, `notes.jsonl`, `exclude`, `stdout`/`stderr`).
3. **Artefacto, não afirmação.** Cada verificação tem um detalhe observado (contagem, hash, exit
   code) e um artefacto cru arquivado ao lado.
4. **O que não é corrido é dito.** O modelo de referência (Ornith 9B) **não** corre neste host:
   mediu-se ~8 min por *completion* (CPU-only). Fica registado como desvio, com a observação feita
   com um modelo pequeno disponível (`G11`) — nunca como se tivesse sido corrido.

## 4. Desvios e limites (registados, não escondidos)

| # | Desvio | Onde |
|---|---|---|
| D1 | O **modelo de referência** (Ornith-1.5-9B Q4_K_M) não é corrido: neste host CPU-only mediu **~8 min por completion** (um turno de 1 passo excedeu 10 min). `G10`/`G11` provam a **mecânica** com guião determinístico; `G11` corre o modelo pequeno disponível (1.5B) como observação. | [G10](G10-cadeia-longa.md), [G11](G11-multi-modelo.md) |
| D2 | `G11` prova a robustez pelo caminho **declarado em texto** (o que um modelo fraco produz), não por dois pesos diferentes do modelo de referência. | [G11](G11-multi-modelo.md) |
| D3 | `G10` usa uma tarefa **pequena e fixa** (um módulo com um bug + um módulo novo com testes) para que o artefacto seja verificável; a barra «equivalência de grau com goose/pi/opencode» continua a ser a bateria `p1`–`p4` (G4). | [G10](G10-cadeia-longa.md) |
| D4 | `G15` cobre o lado do **knudge** correndo os testes do workspace do knudge (`ask --agent`, fonte JSONL) — o binário `kd` não é reimplementado no gate. | [G15](G15-memoria.md) |
| D5 | `scripts/run-server.sh` precisava de `LD_LIBRARY_PATH` (as `libllama*` vivem ao lado do binário); corrigido durante a H12. | [G11](G11-multi-modelo.md) |

## 5. O que os gates encontraram (defeitos reais, corrigidos)

| # | Achado | Correção |
|---|---|---|
| A1 | `--batch --sequential` **não acumulava contexto**: cada turno reabria a sessão em modo `Fresh`, pelo que só o último turno ficava no log (contra "uma sessão, N turnos"). | `batch::turn_config` retoma a sessão a partir do 2.º item (`SessionMode::Resume`); teste `sequential_batch_accumulates_context_in_one_session` |
| A2 | Os **guardrails globais** não eram carregados: `M5` era vacuidade (o piso aditivo não existia). | `facade::builder` e `facade::evaluate_fact` carregam `<user_dir>/guardrails` **antes** das regras do projecto; teste `a_global_guardrail_cannot_be_disabled_by_the_project` |
| A3 | `scripts/run-server.sh` não encontrava as bibliotecas do `llama-server` (falha de arranque). | `LD_LIBRARY_PATH` derivado do directório do binário |
| A4 | **F7/G22**: `cargo doc` com `-D warnings` (novo travão) apanhou um alvo de link explícito redundante (`redundant_explicit_links`) em `crates/ktu-cli/src/instructions.rs` — estava fora do alcance de todos os travões desde o início do projecto. | alvo explícito removido (o rótulo já resolve) |
| A5 | **F5/G20**: a forma plana de `deferred_tools` (`"bash, write_file"`, que a CLI `set` aceita) **falhava** a leitura — o schema tipado exige lista. | o v1 normaliza a forma antiga na leitura e na escrita (e o `schema = 1` fica declarado) |

## 6. Índice dos artefactos

```
artifacts/G10/  g10.jsonl · src-lib.rs · src-media.rs · cargo-test.txt · script.jsonl
artifacts/G11/  g11.jsonl · g11-real-model.jsonl
artifacts/G12/  run.jsonl · repl.jsonl · crate.jsonl
artifacts/G13/  batch-0.jsonl · stderr-lock.txt
artifacts/G14/  exclude.txt · gitattributes.txt · m5.jsonl · doctor.json · doctor-stale.txt · memory-doctor.json
artifacts/G15/  notes.jsonl · session-log.jsonl · stderr-recall.txt · knudge-ai.txt · knudge-corpus.txt
artifacts/G16/  steering.jsonl · projection.txt · steer-dup.jsonl
artifacts/G17/  repl-compact.jsonl · run-compact.jsonl · sigint.jsonl
artifacts/G18/  guardrails.json · skills.txt · plugins.txt
artifacts/G19/  queue-tests.txt · remedy.txt · editing.log · final.txt
artifacts/G20/  doctor-v0.json · config-antes.toml · config-depois.toml · config-recusada.toml · hashes.txt
artifacts/G22/  links.txt · links-broken.txt · links-anchor.txt · doc.txt · ci-knudge.txt · knudge-tests.txt
```

Os artefactos são **desta** corrida: `xtask gates` limpa o directório do gate antes de escrever
(o `g11-real-model.jsonl` só existe depois de correr com `--real-model`).
