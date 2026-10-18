# 0012 — Determinismo: `BTreeMap`, portas para relógio/RNG/FS

- **Status:** aceite
- **Supersedes:** —
- **Superseded by:** —
- **Âncora:** `clippy.toml`, `crates/ktu-core/src/ports/fakes/`

## Contexto

Um agente é um sistema com muitas fontes de não-determinismo (modelo, rede, tempo, ordem de
iteração de mapas). Sem disciplina, os testes passam ou falham conforme a execução — e I6 (*replay
determinístico*) torna-se uma frase em vez de uma garantia.

## Decisão

No núcleo não há fontes de não-determinismo implícitas:

- colecções ordenadas (`BTreeMap`/`BTreeSet`); `HashMap`/`HashSet`/`Rc`/`RefCell`/`Cell`/`LinkedList`
  estão **banidos** por lint no `ktu-core`;
- tempo, aleatoriedade e FS vêm por porta (`Clock`, `Rng`, `Fs`), com *fakes* determinísticos;
- ordenações são estáveis e os desempates explícitos (`(score desc, id asc)`; ranking BM25 com
  desempate por nome);
- aritmética com `checked_*`/`saturating_*` (sem *overflow* dependente do host).

## Alternatives considered

| Alternativa | Porque não |
|---|---|
| `HashMap` no núcleo (mais rápido) | ordem de iteração imprevisível; o custo medido não justifica ([0014](0014-medir-antes-de-optimizar.md)) |
| Ordenar só à saída | o não-determinismo infiltra-se em decisões internas (ex.: qual regra ganha) |
| Aleatoriedade semeada global | um segundo estado global para testar; a porta `Rng` é explícita |
| `SystemTime::now()` onde der jeito | banido por lint na borda excepto no adaptador de relógio |

## Consequências

- O mesmo log + o mesmo provider-gravado dão a mesma conversa (I6), e a busca de memória é
  reprodutível.
- Custo: `BTreeMap` em vez de `HashMap` nos caminhos quentes — medido e aceite.
