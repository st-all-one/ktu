# G14 — Project-bound (init + config)

- **Veredicto:** ✅ **verde** (16/16) · **Artefactos:** `artifacts/G14/` · **Data:** 2026-10-18
- **Critério:** [`GATES.md`](../implementation_phase2/GATES.md) §G14.

## Cenário

Um repositório git novo; `ktu init` (duas vezes), os dois modos de versionamento do conhecimento,
a precedência da configuração e os dois invariantes de confiança (**M3** e **M5**).

```bash
git init -q . && ktu init && ktu init                 # idempotência
ktu init --git-excluded && ktu init --git-tracked     # reversibilidade
ktu config get model                                  # precedência local > global
ktu run ola --provider mock                           # flag > config
ktu run --params '{"prompt":"ola","provider":"mock"}' # params > config
ktu config list --json                                # M3 (auth redigida)
```

## Verificações

| # | Critério | Observado |
|---|---|---|
| 1 | o cenário tem repositório git | `git init` exit 0 |
| 2 | `ktu init` corre | exit 0 |
| 3 | **idempotente**: os ficheiros geridos não mudam | hashes iguais antes/depois (`exclude`, `.gitattributes`, `AGENTS.md`) |
| 4 | marcadores sem duplicação | 1 `# ktu:start` em `.gitattributes`, 1 `/.ktu/` no exclude |
| 5 | por omissão o conhecimento **versiona-se** (só o derivado é excluído) | exclude com `/.agents/memory/index/`, sem `/.agents/` |
| 6 | `--git-excluded` exclui todo o `.agents/` | `/.agents/` no exclude |
| 7 | `--git-tracked` **reverte** ao estado inicial | exclude byte-a-byte igual ao inicial |
| 8 | **M6** precedência: local ganha ao global | `ktu config get model` → `local` |
| 9 | **flag** ganha à config | `--provider mock` corre sem modelo (config dizia `llama`) |
| 10 | **params** ganham à config | `--params '{"provider":"mock"}'` corre sem modelo |
| 11 | **M3** — a chave de auth não ecoa | `config list --json` não contém o segredo |
| 12 | **M5** — guardrail global não é desligável pelo projecto | `allowed:false` no log (o projecto pedia `allow`) |
| 13 | o **`doctor` do projecto** está em dia depois do `init` | `healthy:true` + aponta `ktu memory doctor` |
| 14 | o `init` **não gera a skill do ktu** | `.agents/skills/ktu/SKILL.md` ausente (as convenções vivem na base do prompt) |
| 15 | o `init` **não toca** no `AGENTS.md` do projecto | ficheiro intacto depois do `init` |
| 16 | o diagnóstico da memória tem **porta própria** | `"command":"memory doctor"` + `healthy:true` |

## Artefactos

`exclude.txt`, `gitattributes.txt` (os ficheiros geridos) · `m5.jsonl` (o log do turno negado) ·
`doctor.json` (o diagnóstico em dia) · `memory-doctor.json` (a porta própria da memória).

## Notas

- **M5 ganhou implementação neste gate.** O piso aditivo (guardrails globais) estava adiado (A1 em
  [`ADIADOS`](../implementation_phase2/ADIADOS.md)); sem ele, M5 era vacuidade. Agora
  `facade::builder` e `facade::evaluate_fact` carregam `<user_dir>/guardrails` **antes** das regras
  do projecto — a cadeia só consulta a regra seguinte se a anterior deixar passar, pelo que um
  ficheiro do projecto nunca desliga uma regra global. Travado por
  `a_global_guardrail_cannot_be_disabled_by_the_project` (ktu-core).
- **M3** é fail-closed: um `auth.toml` mal formado **impede o turno** com erro legível (observado
  durante a construção do gate) e a chave nunca entra no log nem no contexto.
- `ktu init` escreve o bloco de *attributes* com marcadores (`# ktu:start`/`# ktu:end`) e as
  exclusões como padrões (sem marcadores), ambos idempotentes.
- **O `doctor` passou a diagnosticar o projecto** (era o diagnóstico do *provider*, apesar de o
  `--help` prometer «layout e conhecimento»): **layout e skill**, cada item com o **remédio**, e o
  comando **aponta** as outras portas (`ktu config doctor`, `ktu memory doctor`,
  `ktu providers doctor`) em vez de as duplicar. Diagnosticava também o bloco de protocolo no
  `AGENTS.md`; esse bloco **deixou de existir** — o `ktu init` não escreve no ficheiro do projecto
  ([ADR 0021](../adr/0021-o-init-nao-escreve-no-agents-md.md)) e o protocolo vive em `ktu prime`.
- **O `ktu init` também deixou de gerar a skill `ktu`**: as convenções passaram para a **base do
  prompt** do agente ([ADR 0022](../adr/0022-convencoes-na-base-do-prompt.md)). Medido num projecto
  novo: `ktu prompt` = **737 bytes** contra 995 (com a skill gerada) — −258 bytes por turno sem
  perder informação. O `doctor` passou a diagnosticar a estrutura que o `init` cria (`layout`,
  `memory_source`, `config`).
- **O diagnóstico da memória ganhou porta própria** (`ktu memory doctor`): estava escondido atrás
  de `ktu doctor --memory`, que sai (uma só porta por área — a superfície mantém-se em 51
  subcomandos e 89 flags).
