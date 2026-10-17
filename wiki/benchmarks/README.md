# Benchmarks — evidência, não fé

Um número sem artefacto não fundamenta decisão (`plan.md` §1.4). Estes benchmarks
medem os caminhos quentes do **núcleo**, de forma determinística (provider e
tools falsos — sem modelo, sem rede).

## Correr

```bash
make bench                                   # → wiki/benchmarks/ktu-bench.json
cargo bench -p ktu-core --bench core         # equivalente
cargo bench -p ktu-core --bench core -- /tmp/out.json   # artefacto noutro sítio
```

Não fazem parte de `make check`: as medições são ruidosas por natureza e não
devem bloquear o portão. Correm-se **a pedido**, antes de decidir optimizar.

## O que mede

| Amostra | O que é |
|---|---|
| `derive_messages_100/1k/10k` | projeção do log → mensagens do provider (o caminho chamado a cada passo) |
| `log_build_1k` | construção de um log sintético de 1000 eventos |
| `turn_1_step` / `turn_8_steps` | um turno completo com 1/8 chamadas de tool (loop + guard + budget) |
| `memory.peak_rss_kib` | pico de RSS do processo |
| `binary.ktu_release_bytes` | tamanho do binário `ktu` de release (se construído) |

## Leitura (corrida de 2026-10-17, Ryzen 5 5500U, rustc 1.99)

| Amostra | mediana |
|---|---|
| `derive_messages_100` | ~6 µs |
| `derive_messages_1k` | ~45 µs |
| `derive_messages_10k` | ~534 µs |
| `log_build_1k` | ~45 µs |
| `turn_1_step` | ~5 µs |
| `turn_8_steps` | ~12 µs |

**Conclusão:** o núcleo custa **microssegundos**; mesmo a projeção de 10 000
eventos fica abaixo de 1 ms. O gargalo de um turno é o **modelo** (inferência
local ou rede), não o loop. Pela prioridade do `plan.md` §2 (performance é
**terciária**), **não há optimização de micro-caminho a fazer** — e há número
para o dizer. Revisitar só se uma medição cruzar um limiar (ex.: projeção > 5 ms
numa sessão real).
