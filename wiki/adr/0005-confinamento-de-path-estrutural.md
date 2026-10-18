# 0005 — Confinamento de path estrutural, nunca regex

- **Status:** aceite
- **Supersedes:** —
- **Superseded by:** —
- **Âncora:** `crates/ktu-tools/src/path.rs`, `crates/ktu-core/src/guard.rs`

## Contexto

O katu chamou uma tool com `/REF/knudge` como caminho (fora do projecto) e repetiu a chamada três
vezes. Um agente que escreve ficheiros precisa de uma fronteira que **não dependa de o modelo ser
razoável** — e que não dependa de *strings* que alguém pode contornar.

## Decisão

O confinamento é **estrutural**: o caminho é resolvido **lexicalmente** (`resolve_within`, sem
`canonicalize` cego) contra o *root*; caminhos absolutos e `..` que escapem são **recusados** com
erro tipado e dica. Não há regex de *allow/deny* de caminhos em lado nenhum.

## Alternatives considered

| Alternativa | Porque não |
|---|---|
| Regex sobre o caminho | contornável (`a/../../etc`, links, unicode); dá falsa segurança |
| Só `canonicalize` | exige que o ficheiro exista (impede criar) e segue links para fora do *root* |
| Sandbox de SO (Landlock/seccomp) | na lista YAGNI do `plan.md` §0: entra só com um sintoma que o exija; o confinamento de *path* resolve o caso real |
| Confiar no *prompt* ("não saias do projecto") | não é garantia, e o erro do katu foi exactamente esse |

## Consequências

- O erro é **accionável**: nomeia o caminho, a razão e o que fazer.
- Vale para todas as tools de escrita/leitura e para o host de plano.
- Custo: caminhos com `..` legítimos (raros) são recusados; a alternativa seria negociar a fronteira.
