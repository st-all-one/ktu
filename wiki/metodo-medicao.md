# Método de medição — medir → A/B → adoptar-ou-reverter

> **Regra:** nenhuma optimização entra sem **número**; nenhum número entra sem **base de
> evidência**. Um lint não substitui uma medição (lição B-01 do katu: 70k linhas e um plano de
> optimização fechado antes de a primeira sessão operar).
>
> Complementa o [dossiê de pendências](pendencias.md) (§3, rejeições com número) e a
> [ADR 0014](adr/0014-medir-antes-de-optimizar.md).

## 1. Os três requisitos de um item

| # | Requisito | O que é |
|---|---|---|
| 1 | **Fórmula** | o que se mede e em que condições (máquina, fixture, iterações, métrica: mediana) |
| 2 | **Artefacto** | o JSON em [`wiki/benchmarks/`](benchmarks/README.md) com os números e o *host* |
| 3 | **Teste que o trava** | o teste que garante que a mudança **não altera comportamento** (o ganho é de tempo, não de semântica) |

Sem o (3), um ganho de performance é indistinguível de uma regressão funcional.

## 2. Base de evidência (`basis`)

Cada número publicado declara de onde vem — é o campo `basis` no artefacto:

| `basis` | Significa | Exemplo |
|---|---|---|
| `measured` | medido **aqui**, com artefacto e *host* | `derive_messages_10k` ≈ 0,5 ms (`ktu-bench.json`) |
| `inferred` | derivado por aritmética de valores medidos | "8 passos ≈ 8 × custo do passo" |
| `provider_reported` | o fornecedor diz (não reproduzido aqui) | latência anunciada de um modelo remoto |
| `unpriced` | **sem número** — a decisão não foi (nem podia ser) medida | GDK rejeitado por ~167 crates e *alpha*, não por tempo |

`unpriced` **não** é uma vergonha: é a declaração honesta de que a decisão se tomou por outra via
(consumidor ausente, determinismo, risco). O que não é aceitável é apresentar `inferred` como
`measured`.

## 3. A/B com recorte cru

1. Correr a **baseline** e guardar o artefacto (`make bench`, com a mesma máquina e a mesma fixture).
2. Aplicar a mudança e correr de novo.
3. Comparar **medianas** do alvo; o ruído do host não interessa (é o recorte que interessa).
4. Registar as duas medições (o `git diff` do JSON é a prova; a wiki guarda a leitura).

## 4. Adoptar-ou-reverter

| Resultado | Acção |
|---|---|
| **≥ 20 %** no alvo | adoptar (e manter o artefacto antes/depois) |
| **remove complexidade** (menos código, menos caminhos) | adoptar mesmo sem ganho de tempo |
| **< 20 % e sem simplificação** | **reverter** e escrever a rejeição **com o número** no [dossiê](pendencias.md) §3 |

A rejeição fica registada com o número que a fundamentou e com o **limiar que a reabre** — para não
voltar a ser discutida a cada fase.

## 5. O caso real (o que já foi decidido assim)

| Decisão | `basis` | Número | Resultado |
|---|---|---|---|
| Sem micro-optimização do núcleo (`decision_01e2183g`) | `measured` | projecção de 10 000 eventos ≈ 0,5 ms; turno de 1 passo ≈ 7 µs | rejeitado — o gargalo é o **modelo**; limiar de reabertura: projecção > 5 ms |
| GDK (`decision_00aw5etj`) | `measured` + `unpriced` | ~167 crates extra sobre 267 | rejeitado — *alpha*, rede não-determinística, MCP na lista YAGNI |
| `BTreeMap` em vez de `HashMap` no núcleo | `unpriced` | — | aceite por **determinismo** (I6), não por tempo; reabre se uma medição puser o `BTreeMap` no caminho quente |

## 6. Correr e ler

```bash
make bench                                  # → wiki/benchmarks/ktu-bench.json
cargo bench -p ktu-core --bench core        # equivalente
```

A leitura da última corrida (com o *host*) está em
[`wiki/benchmarks/README.md`](benchmarks/README.md). Os *benchmarks* **não** entram no `make check`:
medições são ruidosas e não devem bloquear o portão — correm-se **a pedido**, antes de decidir.
