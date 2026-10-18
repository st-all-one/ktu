# knudge — `kd ai` (memória/logs/sessão do agente) + fonte JSONL

- **Estado:** **implementado** (H7, knudge **0.6.2**, `D218`) — `Source` JSONL read-only,
  `ask --agent/--activity`, `kd ai [memory|session-log]`, projeção do log (`ktu memory project`) e
  `ktu-memory` a consumir `parse_jsonl`. **Adiado:** registo de agentes (`.knudge/agents.toml`) —
  só com **≥2** consumidores (YAGNI).
- **Origem:** [estudo `kd ai`](../estudo-kd-ai.md) · [o knudge como eixo de memória](../knudge-eixo-memoria.md); código do `crates/knudge` v0.6.2.
- **Âncora:** `crates/knudge/crates/knudge-core/src/corpus/source.rs`,
  `crates/knudge/crates/knudge-cli/src/commands/ai.rs`, `crates/ktu-memory/src/project.rs`,
  [projeção da sessão](../projecao-sessao.md).

## O que é
Uma **fonte de corpus JSONL read-only** no knudge (`Source::NotesDir` default | `Source::Jsonl`),
que permite ler notas do **formato `{frontmatter, body}`** sem as ter em `notas/*.md` — e uma área
fina `kd ai [memory|session-log] [--add|--init|--anchor|--force]` (default = consulta) sobre essa
fonte. O `session-log` entra **só por projeção** do agente (read-only), nunca como domínio novo.

## Porque interessa
A memória do ktu **já** usa o schema do knudge (`Note`/`Frontmatter`), mas em JSONL — hoje o
`ktu-memory` **reimplementa** a leitura. Uma fonte JSONL elimina essa reimplementação e dá ao
knudge o papel de **eixo único de memória** (memória + logs) para **vários** agentes. Liga a
[G11/121](./knudge-embeddings.md) e a [114](./knudge-store-write.md).

## O que o ktu tem hoje
`ktu-memory` lê `notes.jsonl` → `Vec<Note>` → `Index`/`Graph` do knudge (em memória). O log de
sessão (`.ktu/sessions/<id>.jsonl`) é um **domínio diferente** (eventos de conversa).

## Pré-condições para reconsiderar
- **≥2 agentes** (ktu + goose/pi/opencode) a usar a fonte. Com um só, o ganho **não paga** a
  superfície no knudge.

## Custo e risco
- **Esforço:** baixo (a `Source` JSONL reusa `Note`/`Frontmatter`/`jsonl::json`); a área `kd ai` é
  fina.
- **Risco de não fazer:** o `ktu-memory` continua a reimplementar a leitura; sem eixo único.
- **Riscos a mitigar:** **dupla fonte da verdade** (o log fica no agente; o knudge só o **lê** —
  projeção read-only, derivada); **acoplamento** (o contrato é o **documento**, não o schema
  `Event` do ktu); **scope creep** (não transformar o knudge em *store de logs*).

## Decisão pendente
Avaliar. Ordem: (1) **`Source` JSONL read-only** (o ganho estrutural); (2) **`ask --agent`/`--activity`**
como filtros de consulta (a proveniência já é gravada — D207); (3) **contrato de projeção** (agente →
notas derivadas); (4) **`kd ai`** como área fina sobre a `Source`; (5) **registo de agentes**
(`.knudge/agents.toml`) só com **≥2** consumidores. **Não** acoplar o knudge ao schema do ktu. O
knudge **já** tem o essencial (lock, ids por conteúdo, RRF, proveniência) — isto são **portas e
filtros**, não um motor novo.
