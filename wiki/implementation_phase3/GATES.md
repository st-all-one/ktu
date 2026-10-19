# Gates da Fase 3 — recursos, interacção e travões

> A Fase 2 provou a **mecânica** (G10–G15). A Fase 3 **não** é mais mecânica: cada item tem o
> **gatilho declarado** e um gate que o prova com artefacto. Método:
> [metodo-medicao](../metodo-medicao.md) · Runner: `make gates` (`xtask gates`, o mesmo da Fase 2).
> Data: 2026-10-18.

---

## 0. Princípios

1. **Provar, não afirmar** — cada gate arquiva um artefacto (como a Fase 2).
2. **Gatilho antes do gate** — um item só entra com o sintoma/medição que o exige.
3. **Determinístico** — o mesmo gate, duas vezes ⇒ o mesmo quadro.
4. **Sem regressão** — os G10–G15 continuam verdes; o F2 mexe no REPL, pelo que **G12** (log
   byte-idêntico `run`/REPL/crate) é o teste de regressão obrigatório.

---

## G16 — *Steering* durável (M7) — ✅ **verde** (9/9, 2026-10-18)

[Relatório](../gates/G16-steering.md) · artefactos em `wiki/gates/artifacts/G16/`.

**Prova:** uma mensagem admitida é entregue **exactamente uma vez**, mesmo com *crash*/*resume* e
com o front a repetir o pedido.

- **Cenário:** admitir (`SteerQueued`) → matar o processo antes da entrega → `resume`; e reenviar
  o mesmo `id` depois de entregue.
- **Aceite:** a mensagem aparece **uma** vez na projeção; ids distintos com o mesmo texto ⇒ duas
  mensagens; o log mantém I1/I2.
- **Artefacto:** `steering.jsonl` (log da sessão) + projeção com contagem de `Message::User`.

## G17 — REPL interactivo — ✅ **verde** (10/10, 2026-10-18)

[Relatório](../gates/G17-repl.md) · artefactos em `wiki/gates/artifacts/G17/`.

**Prova:** o REPL é usável sem lógica própria (M4) e sem *raw mode* obrigatório.

- **Cenário:** `/compact` no REPL vs `--compact` na CLI; SIGINT durante um turno; REPL em *pipe*.
- **Aceite:** `/compact` ⇒ a **mesma** projeção que a CLI; SIGINT ⇒ o processo **não** morre e o
  log fecha com `StopReason::Cancelled`; em *pipe* o log continua **byte-idêntico** ao do `run`
  (**G12**).
- **Artefacto:** `repl-compact.jsonl`, `run-compact.jsonl` (diff vazio) + `sigint.jsonl`.

## G18 — Recursos globais e piso visível (M8) — ✅ **verde** (10/10, 2026-10-18)

[Relatório](../gates/G18-recursos.md) · artefactos em `wiki/gates/artifacts/G18/`.

**Prova:** a precedência é declarada, o piso é aditivo e **tudo o que o runtime usa é visível**.

- **Cenário:** mesma skill/regra/receita nos dois níveis; plugin global + local; `ktu guardrails
  list --json`.
- **Aceite:** local **ganha** por nome (skills/receitas); guardrails **somam** e o do projecto
  **não** desliga o global; o `list` mostra `origem` para **todas** as regras efectivas.
- **Artefacto:** `guardrails.json` (com origem), `skills.txt`, `plugins.txt`.

## G19 — Edição sob concorrência — ✅ **verde** (8/8, 2026-10-18)

[Relatório](../gates/G19-edicao.md) · artefactos em `wiki/gates/artifacts/G19/`.

**Prova:** mutações ao mesmo ficheiro **não perdem dados**.

- **Cenário:** N `write`/`edit` concorrentes ao mesmo caminho (com paralelismo de tools ligado).
- **Aceite:** o ficheiro final é **um** dos escritos, sem intercalar bytes; caminhos distintos
  correm em paralelo; o remédio da regra aparece na mensagem.
- **Artefacto:** ficheiro final + log das tools + mensagem de recusa com remédio.

## G20 — Migração de config — ✅ **verde** (7/7, 2026-10-18)

[Relatório](../gates/G20-migracao.md) · artefactos em `wiki/gates/artifacts/G20/`.

**Prova:** migrar é **idempotente** e a leitura **não destrói**.

- **Cenário:** config v0 (sem versão) → v1 nos dois níveis; correr 2×; a forma plana antiga de
  `deferred_tools` na leitura; versão futura (v3).
- **Aceite:** o `doctor` mostra a versão sem escrever; a leitura aceita a forma antiga e deixa o
  ficheiro **intacto**; 2× ⇒ hash igual; v3 ⇒ erro legível e **nenhuma** escrita.
- **Artefacto:** `doctor-v0.json` + `config.toml` antes/depois/recusada + hashes.

## G21 — `ktu self setup` — ⛔ **sem gate** (item fechado sem plano)

O item **F6** foi fechado sem plano ([porquê](./F6-self-setup.md)): o `ktu init` já entrega o que
tem consumidor (bloco de protocolo no `AGENTS.md` + skill) e o ktu não tem servidor MCP. **Um item
cujo gatilho não disparou não tem gate** — a pendência (**D3**) fica no
[dossiê](../pendencias.md) §2 com o critério de reabertura. O número **G21** fica por usar (a
numeração dos gates é estável; não se reutiliza).

## G22 — Travões (links/doc e knudge) — ✅ **verde** (6/6, 2026-10-18)

[Relatório](../gates/G22-travoes.md) · artefactos em `wiki/gates/artifacts/G22/`.

**Prova:** as lacunas L1/L3 deixam de ser lacunas.

- **Cenário:** link deliberadamente quebrado; âncora inexistente; `cargo doc`; o gate do knudge
  na CI.
- **Aceite:** `check-links` falha no link quebrado (com o par ficheiro/alvo) e passa na árvore
  actual (0 quebrados, 0 avisos); a âncora inexistente é aviso, não erro; o doc não tem avisos;
  a CI corre o gate do knudge **e** a suite do knudge passa.
- **Artefacto:** saída do `check-links` (falha e sucesso) + o passo do workflow + os 22 resumos
  da suite do knudge.

---

## Tabela-resumo

| Gate | Foco | Condição-chave | Item |
|---|---|---|---|
| **G16** | *steering* durável ✅ 9/9 | entrega exactamente uma vez (M7) | F1 |
| **G17** | REPL interactivo ✅ 10/10 | Ctrl-C sem matar, sem regressão no G12 | F2 |
| **G18** | recursos globais ✅ 10/10 | precedência + piso visível (M8) | F3 |
| **G19** | edição concorrente ✅ 8/8 | sem perda de dados | F4 |
| **G20** | migração de config ✅ 7/7 | idempotente, leitura não-destrutiva | F5 |
| ~~G21~~ | ~~`ktu self setup`~~ | **sem gate** (F6 fechado sem plano) | — |
| **G22** | travões ✅ 6/6 | links/doc + knudge na CI | F7 |

**Gate de saída da Fase 3:** os gates dos itens **abertos** verdes, com artefactos arquivados em
`wiki/gates/artifacts/<gate>/`, + `make check`, `make gates` e `make supply-chain` verdes.
Um item cujo **gatilho não disparou** não tem gate — fica no
[dossiê](../pendencias.md) com a condição declarada.

**Estado (2026-10-18):** os gates **G16–G20** e **G22** estão verdes (**12 gates, 94
verificações**); `make check`, `make gates` e `make supply-chain` verdes. **F5** foi feita **sem o
gatilho declarado** (o mecanismo de versionamento é pré-condição de qualquer migração futura —
desvio registado na [ADR 0020](../adr/0020-schema-de-config-versionado-com-leitura-nao-destrutiva.md));
**F6** foi **fechado sem plano** (sem gate: o `ktu init` já cobre o que tem consumidor). A Fase 3
está **fechada** com **todos** os itens resolvidos.
