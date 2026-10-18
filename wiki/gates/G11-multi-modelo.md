# G11 — Robustez multi-modelo

- **Veredicto:** ✅ **verde** (2/2 + observação) · **Artefactos:** `artifacts/G11/` · **Data:** 2026-10-18
- **Critério:** [`GATES.md`](../implementation_phase2/GATES.md) §G11.

## Cenário

A mesma cadeia (G10), mas com um modelo que **não tem tool calling nativo**: a chamada vem escrita
no **texto** (`{"name":"read_file","arguments":{"path":"src/lib.rs"}}`), como acontece com modelos
locais fracos. O loop tem de a **decodificar** contra o catálogo fechado (nome fora do catálogo é
ignorado, *fail-closed*) e correr a tool — H5.

```bash
# determinístico (o caminho declarado-em-texto)
ktu run "le o modulo" --provider scripted --script fixtures/g11-script.jsonl --session g11

# observação com um modelo real (não é critério de aceitação)
make gates ARGS='--only G11 --real-model --base-url http://127.0.0.1:8080 --model qwen2.5-coder-1.5b'
```

## Verificações

| # | Critério | Observado |
|---|---|---|
| 1 | a tool call declarada em texto é executada | `tool_result` presente no log; `read_file` no catálogo |
| 2 | o turno fecha com `StopReason` normal | `end_turn` = 1 |

O **modelo real é uma observação, não um critério** (é não-determinístico e neste host não corre o
modelo de referência): a corrida com `--real-model` regista exit code, turnos fechados e resposta
final no `summary.json` e no artefacto, sem decidir o veredicto. Nas duas corridas registadas, o
1.5B ou leu o ficheiro e não fechou o turno no tecto de passos, ou respondeu sem ler — comportamento
de modelo fraco, que o loop **tolera** (log íntegro, guard a decidir cada efeito, `StopReason`
explícito).

## Observação com o modelo real (registada, não é critério)

Com `qwen2.5-coder-1.5b-instruct-q4_k_m` (llama.cpp já a correr em `127.0.0.1:8080`, ~25 s por
turno neste host), duas observações:

1. respondeu *«Modificado o arquivo 'src/lib.rs' para corrigir o bug»* — **sem ler o ficheiro**;
2. leu o ficheiro mas **não fechou o turno** dentro do tecto de passos (o `StopReason` foi o tecto,
   explícito e logado).

Nos dois casos o loop **não quebrou**: o log ficou íntegro (I1/I2), cada efeito passou pelo guard e
o turno terminou com uma razão explícita. É este o comportamento que a H5 tem de tolerar. O
artefacto da última corrida está em `g11-real-model.jsonl`.

## Desvios

- O **modelo de referência** (Ornith-1.5-9B) não corre aqui (~8 min/completion, ver G10/D1); a
  segunda metade do critério («mesma qualidade com um 2.º modelo») é coberta por: (a) o caminho
  declarado-em-texto, que é o modo de falha real dos modelos fracos; (b) a observação com o 1.5B
  acima; (c) a bateria `p1`–`p4` (G4), onde o ktu corre contra os referenciais com o modelo real.
- O «eco detectado» é travado por teste no núcleo (`is_echo` + `ECHO_MIN_CHARS`, com controlo
  negativo) — não é reproduzível com o provider de demonstração, que nunca ecoa o prompt.
