# G14 — Project-bound (init + config)

- **Veredicto:** ✅ **verde** (12/12) · **Artefactos:** `artifacts/G14/` · **Data:** 2026-10-18
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

## Artefactos

`exclude.txt`, `gitattributes.txt` (os ficheiros geridos) · `m5.jsonl` (o log do turno negado).

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
