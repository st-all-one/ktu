# 0008 — Provider fino por dialeto; GDK não

- **Status:** aceite
- **Supersedes:** —
- **Superseded by:** —
- **Âncora:** `crates/ktu-provider/src/wire.rs`, `crates/ktu-cli/src/providers.rs`

## Contexto

O ktu tem de falar com um modelo local (`llama-server`) e, quando o host não aguenta, com um
provider remoto. O `REF/goose` tem um *SDK* de providers (GDK) com dialetos prontos — a tentação é
adoptá-lo. A avaliação (G9) mostrou `goose-sdk` em `alpha` e `goose-providers` a puxar ~167 crates
extra, incluindo `rmcp`/MCP (lista YAGNI), com rede não-determinística de testar.

## Decisão

Um **provider fino por dialeto**, escrito aqui e testado com o *test double* HTTP próprio: o
built-in é o `llama-server` (OpenAI-compatible local) e o remoto entra atrás de uma flag
(`--provider opencode-go`, também OpenAI-compatible). Sem GDK, sem MCP, sem segundo agente.

## Alternatives considered

| Alternativa | Porque não |
|---|---|
| GDK (`goose-sdk` + `goose-providers`) | alpha, ~167 crates extra (com MCP), não-determinístico de testar, e traria tipos de mensagem de outro agente |
| SDK oficial por fornecedor (Anthropic/OpenAI) | um dialeto por dependência; o caminho OpenAI-compatible cobre a necessidade actual |
| Um *proxy* externo (LiteLLM e afins) | muda a fronteira de confiança para fora do binário e acrescenta um processo |
| MCP como fonte de tools | na lista YAGNI; as tools são locais e guardadas por [0004](0004-guardrails-em-tres-camadas.md) |

## Consequências

- O *wire* é testável byte-a-byte (goldens) e o provider é substituível por um `FakeProvider`.
- Quando surgir necessidade real de outro dialeto, a decisão é **um provider fino novo** — não
  adoptar o GDK.
- Custo: cada dialeto novo é trabalho nosso; em troca, o binário mantém ~267 crates e um só agente.
