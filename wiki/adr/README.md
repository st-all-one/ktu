# ADRs — decisões com alternativas

Cada decisão arquitectural do ktu vive numa **ADR imutável**: `NNNN-slug.md` (4 dígitos).
Uma ADR **não se edita** — quando a decisão muda, escreve-se outra e ligam-se as duas pontas
com `Supersedes`/`Superseded by`. Uma decisão sem o que perdeu convida a re-litigá-la.

## Contrato (imposto por `make check` → `xtask check-adr`)

1. Nome `NNNN-slug.md`, id único.
2. Título `# NNNN — <título>`.
3. `**Status:** aceite | substituída | proposta` + `**Supersedes:**` / `**Superseded by:**`
   (`—` quando não há; as duas pontas têm de concordar).
4. Quatro secções obrigatórias: `## Contexto`, `## Decisão`, **`## Alternatives considered`**,
   `## Consequências`. A terceira é a razão de existir deste directório.
5. Este índice lista todas as ADRs.

Incidentes reais vivem em [`../postmortems/`](../postmortems/README.md) (mesmo gate).

## Quando escrever uma ADR

- A decisão **fecha uma alternativa plausível** (arquitectura, contrato, processo com custo).
- A decisão **vai ser questionada** daqui a três meses (senão vive num comentário).
- Altera um invariante (I1–I17), um gate, ou a superfície pública.

Não é para tudo: bugs, estilo e nomes ficam no código e nos testes. ADRs curtas (1 página).

## Índice

| # | Decisão | Status |
|---|---|---|
| [0001](0001-kernel-puro-com-portas.md) | Kernel puro com portas; efeitos só na borda | aceite |
| [0002](0002-duas-threads-e-front-desacoplado.md) | Duas threads (kernel + tools) e front desacoplado | aceite |
| [0003](0003-log-append-only-e-a-fonte-da-verdade.md) | O log append-only é a fonte da verdade (I3) | aceite |
| [0004](0004-guardrails-em-tres-camadas.md) | Guardrails em três camadas, fail-closed | aceite |
| [0005](0005-confinamento-de-path-estrutural.md) | Confinamento de path estrutural, nunca regex | aceite |
| [0006](0006-read-only-estrutural.md) | Read-only estrutural (subagentes e modo plano) | aceite |
| [0007](0007-memoria-jsonl-e-knudge-como-eixo-unico.md) | Memória JSONL e o knudge como eixo único | aceite |
| [0008](0008-provider-fino-sem-gdk.md) | Provider fino por dialeto; GDK não | aceite |
| [0009](0009-fachada-dentro-do-ktu-core.md) | A fachada de incorporação vive no `ktu-core` | aceite |
| [0010](0010-cli-e-o-front-primario.md) | A CLI é o front primário | aceite |
| [0011](0011-limite-de-400-linhas-e-firewall-mecanico.md) | Limites mecânicos: 400 linhas, camadas, superfície | aceite |
| [0012](0012-determinismo-no-nucleo.md) | Determinismo: `BTreeMap`, portas para relógio/RNG/FS | aceite |
| [0013](0013-sem-unsafe-unwrap-panic.md) | Sem `unsafe`/`unwrap`/`panic`; `allow` com `reason` | aceite |
| [0014](0014-medir-antes-de-optimizar.md) | Medir antes de optimizar (adoptar-ou-reverter) | aceite |
| [0015](0015-yagni-com-criterio-de-reabertura.md) | YAGNI com critério de reabertura | aceite |
| [0016](0016-h10-adiar-receitas-ricas-tool-search-e-agents-na-leitura.md) | H10: adiar receitas ricas, `tool_search` e `AGENTS.md` na leitura | substituída |
| [0017](0017-implementar-a9-a11-antes-da-h11.md) | Implementar A9–A11 antes da H11 | aceite |
| [0018](0018-gates-da-fase-2-por-guiao-deterministico.md) | Gates da Fase 2 por guião determinístico | aceite |
