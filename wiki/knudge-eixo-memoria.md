# O knudge como eixo único de memória (agent-agnostic)

> Visão aprovada: **uma só fonte de verdade estruturada no knudge**, para que qualquer agente
> (ktu, goose, pi, opencode, …) se integre no **mesmo padrão**, mantendo o domínio de memória,
> conhecimento e tarefas **independente do agente** — com consulta **inteligente** (não recuperação
> sequencial de logs) e **concorrência multi-agente** natural. Data: 2026-10-17.
>
> Irmãos: [estudo `kd ai`](./estudo-kd-ai.md) · [CLI e configuração](./cli-e-config.md) ·
> [revisão do knudge](./revisao-knudge.md).

---

## 0. Princípio

> **O agente é substituível; a memória não.** O knudge detém o **domínio durável** (conhecimento,
> tarefas, decisões, riscos); o agente é um **produtor** desse domínio. Trocar de agente **não**
> troca o padrão, nem a memória, nem a consulta.

Três corolários:
1. **Um só standard** — o schema do knudge (`type`/`scope`/`statement`/`body`/`anchors`/`tags`/
   `provenance`), **independente** do agente.
2. **Estruturado, não log** — o que entra é **conhecimento estruturado** (notas/claims), não um
   despejo de log. A consulta é **inteligente** (BM25 + âncoras + grafo + RRF), não sequencial.
3. **Multi-agente por construção** — vários agentes escrevem em paralelo, em segurança.

---

## 1. O que o knudge já oferece (medido)

| Propriedade | Mecanismo | Ficheiro |
|---|---|---|
| **Concorrência** | lock advisory **por ficheiro** (`O_CREAT\|O_EXCL`, stale 30 s, reclaim por rename, retry+jitter, ordem container→nota) | `store/lock.rs` (D23–D25) |
| **Commit seguro** | **nota primeiro, evento depois**; crash = nota sem evento (recuperável) | `store/commit.rs` (D21) |
| **Ids determinísticos** | id = hash(`type` + separador + statement normalizado) — dedup sob merge | `schema/id.rs` (D26/D28/D95) |
| **Merge concorrente** | `.gitattributes` `merge=union` no log append-only; o id do evento torna-o seguro | `git/attributes.rs` (D26/D28/D31) |
| **Consulta inteligente** | BM25 + âncoras + grafo + **fusão RRF** (k=60); canais ausentes degradam | `retrieval/` (D39/D41/D81) |
| **Proveniência** | `entity`/`activity`/`agent` (PROV-lite) — **quem** produziu a nota | `schema/provenance.rs` (D207) |
| **Corpus de leitura única** | lê `notas/` uma vez → `Index` + `Graph` (paralelo, ordem determinística) | `corpus/mod.rs` |
| **Fachada de incorporação** | `Knudge::builder().knowledge_dir(…).open()` | `knudge.rs` (D214) |

**Conclusão:** o knudge **já é** o eixo agent-agnostic. A `provenance.agent` já distingue
produtores; o lock + ids por conteúdo já tornam vários agentes seguros; o RRF já é consulta
inteligente. **Falta menos do que parece** — falta o **contrato** e a **porta de entrada** para
agentes que não escrevem `.md`.

---

## 2. O contrato de integração (o standard)

Um agente integra-se emitindo **notas** no schema do knudge. O contrato mínimo:

```yaml
type:      fact | decision | error | risk | question | def | snippet | link | meta | task
scope:     <epic_id>            # opcional; agrupa o trabalho
statement: <≤120 escalares>     # UMA afirmação auto-contida (chave do retrieval)
body:      <contexto>           # Por quê / Evidência / Consequência (opcional)
anchors:   [src/**, …]          # liga ao código (glob)
tags:      [ … ]
provenance:
  agent:    ktu | goose | pi | opencode | <membro>
  activity: write | learn | import | derive
  entity:   <o que a nota descreve>   # opcional
status:    active | in_progress | blocked | closed | superseded | forgotten
```

**Regras do standard** (as mesmas do `kd`):
- **uma** afirmação por nota (não juntar);
- statement curto e auto-contido; corpo só quando o statement não permite agir;
- **nunca** inventar id (deriva do conteúdo);
- `ask` antes de `write` (dedup: <0.75 cria · 0.75–0.92 funde · ≥0.92 rejeita);
- âncoras ligam ao código; `provenance.agent` identifica o produtor.

> **O standard é o mesmo para todos os agentes** — é isto que garante "domínio total dentro do
> mesmo padrão independente do agente".

---

## 3. As três vias de integração de um agente

| Via | Quando | Como | Fonte da verdade |
|---|---|---|---|
| **Nativa** | o agente fala o schema (ktu hoje) | escreve notas via fachada/CLI/MCP | **knudge** (as notas) |
| **Adaptada** | o agente tem store próprio (JSONL) | **`Source` JSONL read-only** lê o store e projeta em notas | **agente** (read-only) |
| **Projetada** | o agente tem log cru (sessão) | o agente **projeta** eventos em notas **derivadas** | **agente** (o log); knudge só indexa |

**Regra de ouro:** o knudge **indexa** (derivado); o agente **possui** a fonte. Nunca duas
verdades — o log do agente continua a ser dele (I3); o knudge lê-o/projeta-o **read-only**.

- **ktu (nativa):** o `ktu-memory` já escreve notas no schema do knudge.
- **ktu (adaptada):** `.agents/memory/notes.jsonl` é lido pela `Source` JSONL (elimina a
  reimplementação no `ktu-memory`).
- **ktu (projetada):** `.ktu/sessions/<id>.jsonl` → notas **derivadas** (ex.: uma por decisão/turno
  com `type=meta`, `provenance.activity=derive`), para pesquisa **entre sessões**.

---

## 4. Consulta inteligente (não sequencial)

O pedido é explícito: **não** recuperação sequencial de logs. O caminho:

1. **Estruturar à entrada** — o agente emite notas (afirmações + âncoras), não linhas de log.
2. **Fundir sinais à saída** — `ask` usa BM25 (statement + corpo) + âncoras (código) + grafo
   (arestas) + semântico (embeddings) via **RRF**; pesos configuráveis (`recall.lexical_weight`,
   `anchor_weight`, `semantic_weight`, `ppr_weight`).
3. **Filtrar por proveniência** — `ask --agent ktu` (quem), `--activity`, `--type`, `--anchor`,
   `--since/--until` — permite "o que o goose aprendeu sobre X" sem ler logs.
4. **Derivar confiança** — `lifecycle` (Beta, shelf-life) e `task_confirmation` já priorizam.

**Consequência para o agente:** o seu log **não** é consultado linha a linha; é **projetado** em
notas e consultado por **significado + âncoras + grafo**.

---

## 5. Concorrência multi-agente (o que já há; o que falta)

### Já há
- **Lock por ficheiro** → dois agentes a escrever **notas diferentes** não conflituam; na **mesma**
  nota, um espera (ou funde por dedup).
- **Ids por conteúdo** → o mesmo facto de dois agentes **deduplica** (funde), não duplica.
- **Commit nota→evento** + **union merge** → o log de eventos sobrevive a merge concorrente.
- **Proveniência** → cada nota sabe **quem** a produziu.

### Falta (pequeno)
1. **`Source` JSONL read-only** — a porta de entrada para agentes com store próprio (o caso JSONL).
2. **Filtro `ask --agent`** — hoje a proveniência é **gravada**; convém **consultá-la** como filtro
   de primeira classe (se ainda não estiver exposto).
3. **Registo de agentes** (opcional) — `.knudge/agents.toml` (nome + store + padrão) para o `kd ai`
   saber o que ler; **sem** acoplar schemas.
4. **Projeção documentada** — o contrato de como um agente projeta o seu log em notas derivadas.

> **Nota:** nenhum destes cria um segundo motor; são **portas e filtros** sobre o que existe.

---

## 6. O que falta construir (ordem)

1. **`Source` JSONL read-only** no `knudge-core` (o ganho estrutural; reusa `Note`/`Frontmatter`).
2. **`ask --agent`/`--activity`** como filtros de consulta (a proveniência já é gravada).
3. **Contrato de projeção** (doc) — como um agente projeta o seu log em notas derivadas.
4. **`kd ai`** como área fina (posicional, default consulta) sobre a `Source`.
5. **Registo de agentes** (`.knudge/agents.toml`) — **só** com ≥2 agentes.

---

## 7. Checklist — integrar um novo agente

- [ ] Emitir notas no **schema do knudge** (uma afirmação por nota; `statement` curto).
- [ ] Preencher `provenance.agent` (e `activity`).
- [ ] Ligar ao código com `anchors` (glob).
- [ ] `ask` antes de `write` (dedup); fechar tarefas com `--outcome`.
- [ ] Se tiver store próprio: expor um **JSONL** `{frontmatter, body}` para a `Source`.
- [ ] Se tiver log cru: **projetar** em notas derivadas (`activity=derive`), read-only.
- [ ] Nunca duplicar a fonte da verdade (o log fica no agente).

---

## 8. Veredicto

| Pergunta | Resposta |
|---|---|
| O knudge já é o eixo agent-agnostic? | **Quase** — lock, ids por conteúdo, RRF e proveniência já lá estão. |
| O que falta? | A **porta** (`Source` JSONL), o **filtro** (`ask --agent`), o **contrato** (projeção) e a **área** (`kd ai`). |
| Custo? | **Baixo** — portas e filtros, nenhum motor novo. |
| Risco? | Acoplar o knudge a schemas de agentes (evitar: o contrato é o **documento**). |
| Ganho? | **Estratégico** — a memória deixa de ser do agente e passa a ser do **projecto/equipa**. |

> **Em uma frase.** O knudge já tem os **mecanismos** (concorrência, fusão, proveniência); falta-lhe
> a **porta** para os agentes e o **contrato** do standard — e com isso a memória, o conhecimento e
> as tarefas ficam **independentes do agente**, que é exactamente o objectivo.

---

## 9. Estado da implementação (H7)

Implementado no knudge **0.6.2** (decisão `D218`) e no ktu (H7).

| Peça | Onde | Estado |
|---|---|---|
| **`Source` JSONL read-only** | `knudge-core::corpus::Source` (`NotesDir` \| `Jsonl`) + codec `Note::to_jsonl`/`from_jsonl` | ✅ |
| **Filtros de proveniência** | `kd ask --agent/--activity` (e `--rank`), no `Filter`/`Meta` e no `.idx/` | ✅ |
| **Contrato de projeção** | [`fontes.md`](../crates/knudge/wiki/specs/fontes.md) (knudge) + [`projecao-sessao.md`](projecao-sessao.md) (ktu) | ✅ |
| **`kd ai [memory\|session-log]`** | `knudge-cli::commands::ai` (área fina, posicional, default consulta) | ✅ |
| **`ktu-memory` consome a `Source`** | `knudge_core::corpus::parse_jsonl` — o ktu **não** reimplementa o codec nem a tolerância | ✅ |
| **Projecção do log de sessão** | `ktu memory project` → `.agents/memory/session-log.jsonl` (`activity=derive`) | ✅ |
| **Proveniência nas escritas do ktu** | `agent = ktu`, `activity = write` | ✅ |
| **Registo de agentes** (`.knudge/agents.toml`) | — | ⏳ **adiado** (YAGNI: só com ≥2 consumidores) |

Prova: `make check` verde no ktu **e** no knudge; `kd ai session-log` devolve as notas projetadas;
`kd ask --agent ktu` filtra por proveniência.
