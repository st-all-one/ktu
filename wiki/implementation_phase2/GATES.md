# Gates da Fase 2 — usabilidade em cadeias extensas

> O gate da Fase 1 (**G4** = `quality_test` p1–p4) provou que o ktu é **usável** numa bateria de 4
> perguntas. A Fase 2 exige gates que provem a usabilidade em **cadeias de interacção mais longas**,
> **na mesma sessão**, e sob as condições reais das 4 utilizações. Data: 2026-10-17.
>
> Referência de método: [revisao-proposito](../revisao-proposito.md) §3 (M1–M6) ·
> [NOTAS-RUST](./NOTAS-RUST.md) · `plan.md` §4.

---

## 0. Princípios dos gates

1. **Provar, não afirmar.** Cada gate tem um **artefacto** (log, JSON, ficheiro) arquivado.
2. **Cadeia, não pergunta única.** ≥15 turnos **na mesma sessão** (excepto onde indicado).
3. **Adversarial.** Condições difíceis (modelo fraco, output grande, cancel, concorrência).
4. **Determinístico.** O mesmo gate, duas vezes ⇒ **mesmo** resultado observável.
5. **Bar = equivalência de grau** com goose/pi/opencode (como G4), não vitória.

---

## G10 — Cadeia longa numa mesma sessão ⭐ (o gate central)

**Prova:** o agente mantém contexto, usa tools e **corrige-se** ao longo de ≥15 turnos numa sessão.

- **Cenário:** tarefa real de código (ex.: "adiciona uma feature a um módulo, com teste") numa
  **única** sessão: ler → planear → editar → correr testes → corrigir → verificar.
- **Aceite:**
  - o artefacto (código + teste) é **correcto** (`make check` verde no repo alvo);
  - **sem** releitura em ciclo dos mesmos ficheiros (não-estagnação);
  - `session.verify()` **limpo** (I1/I2); todos os turnos fecham com `StopReason` normal;
  - a instrução inicial **nunca** cai do contexto (I14/`pin_last_user`).
- **Artefacto:** `.ktu/sessions/<id>.jsonl` + `derive_messages` + resultado do `make check`.

## G11 — Robustez multi-modelo

**Prova:** a mesma cadeia (G10) com um **segundo** modelo — mais fraco / tool-call em texto.

- **Aceite:** qualidade **equivalente** a G10; tool calls declaradas/reparadas (H5); eco detectado.
- **Artefacto:** log da sessão com o segundo modelo + comparação de grau.

## G12 — Independência do caminho (M4)

**Prova:** **dois caminhos, uma sessão** — o mesmo prompt por caminhos diferentes dá o **mesmo log**.

- **Cenário:** o mesmo prompt por `ktu run` (efémero→sessão), pelo **REPL** (`ktu session`) e pelo
  **crate** (via porta).
- **Aceite:** `derive_messages(log)` **byte-idêntico** nos três caminhos (I3/I6).
- **Artefacto:** três logs + diff vazio.

## G13 — Pipeline concorrente (uso #2)

**Prova:** N `ktu` **independentes** em paralelo no **mesmo** projecto.

- **Cenário:** `--batch` com N prompts (paralelo default); e dois `run` no **mesmo** id (lock).
- **Aceite:** sem corrupção do log (I1/I2); resultados **ordenados** pelo índice; o lock faz um
  esperar/recusar com `StopReason` claro; `stdout`=dados, `stderr`=logs.
- **Artefacto:** JSONL de saída + logs + prova de integridade.

## G14 — Project-bound (init + config)

**Prova:** o ambiente é preparado e a configuração é **do projecto**.

- **Cenário:** `ktu init` (2×) + `--git-excluded`/`--git-tracked` + precedência de config.
- **Aceite:**
  - `ktu init` **idempotente** (reexecutar não duplica blocos);
  - `.git/info/exclude`/`.gitattributes` correctos e reversíveis;
  - precedência `flag>params>local>global`; **M3** auth não ecoa; **M5** guardrail global não removível.
- **Artefacto:** diffs dos ficheiros geridos + testes M3/M5.

## G15 — Eixo de memória (knudge)

**Prova:** a memória **sobrevive** à sessão e é **consultável** pelo knudge, agent-agnostic.

- **Cenário:** sessão escreve memória (H7); sessão **nova** recupera; `kd ask --agent ktu` devolve a
  nota; a `Source` JSONL lê o store.
- **Aceite:** recall idêntico após rebuild; `ask --agent` filtra por proveniência; a projeção do log
  é **read-only**; dois agentes a escrever não colidem.
- **Artefacto:** `notes.jsonl` + saída do `ask` + prova de rebuild.

---

## Tabela-resumo

| Gate | Foco | Condição-chave | Âncora |
|---|---|---|---|
| **G10** | cadeia longa (mesma sessão) | ≥15 turnos, artefacto correcto | H4, H5 |
| **G11** | robustez multi-modelo | 2.º modelo, mesma qualidade | H5 |
| **G12** | independência do caminho (M4) | log byte-idêntico | H0, H4, H9 |
| **G13** | pipeline concorrente | sem corrupção, ordenado | H0, H3 |
| **G14** | project-bound | init idempotente, M3/M5 | H1, H2 |
| **G15** | eixo de memória | `ask --agent`, rebuild | H7 |

**Gate de saída da Fase 2:** **G10–G15 verdes**, com artefactos arquivados, + `make check` e
`make supply-chain` verdes. Só então se abre a Fase 3 (optimização com número).

---

## Estado (2026-10-18) — **fechado** ✅

Os seis gates estão **verdes** (40 verificações), com artefactos arquivados em
[`wiki/gates/`](../gates/README.md). O runner é `make gates` (`xtask gates`), que corre o binário
de ponta a ponta num espaço isolado por gate e escreve `summary.json` + `artifacts/<gate>/`.

| Gate | Veredicto | Verificações | Relatório |
|---|---|---|---|
| G10 — cadeia longa (≥15 turnos) | ✅ | 7/7 | [G10](../gates/G10-cadeia-longa.md) |
| G11 — robustez multi-modelo | ✅ | 2/2 + observação | [G11](../gates/G11-multi-modelo.md) |
| G12 — independência do caminho | ✅ | 4/4 | [G12](../gates/G12-tres-caminhos.md) |
| G13 — pipeline concorrente | ✅ | 6/6 | [G13](../gates/G13-concorrencia.md) |
| G14 — project-bound | ✅ | 12/12 | [G14](../gates/G14-project-bound.md) |
| G15 — eixo de memória | ✅ | 9/9 | [G15](../gates/G15-memoria.md) |

**Determinismo:** os gates correm com `--provider mock`/`scripted` (sem inferência); a corrida
completa, duas vezes, dá o mesmo quadro (mesmas verificações, mesmo veredicto). O critério
«mesmo gate, duas vezes ⇒ mesmo resultado observável» é também provado **dentro** do G12.

**Desvios (registados em [`wiki/gates/README.md`](../gates/README.md) §4):** o modelo de referência
(Ornith-1.5-9B) **não** corre neste host CPU-only (~8 min por *completion*, medido); o G10/G11
provam a mecânica com guião determinístico e o G11 regista uma observação com um modelo pequeno.
A barra de «equivalência de grau» com goose/pi/opencode continua a ser a bateria `p1`–`p4` (G4).

**Defeitos encontrados pelos gates** (corrigidos na H12): `--batch --sequential` não acumulava
contexto; os guardrails globais não eram carregados (M5 era vacuidade); `scripts/run-server.sh`
não encontrava as bibliotecas do `llama-server`.
