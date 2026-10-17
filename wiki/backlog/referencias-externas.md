# Referências externas

- **Estado:** candidata (do `opencode`) — **avaliar**.
- **Origem:** `REF/opencode/.opencode/opencode.jsonc` (`references`);
  `wiki/revisao-opencode.md` §2.7.
- **Âncora:** `.agents/`, `crates/ktu-core/src/prompt.rs`.

## O que é
A configuração declara **referências** (repo/path + descrição) que o agente pode consultar
— ex.: `effect` → repo `Effect-TS/effect-smol`; `opencode-local` → `~/.local/share/opencode`.

## Porque interessa
Dar ao agente **ponteiros** para recursos externos (docs, código de referência) é simples
e útil. É **dado**, não efeito; o agente decide se consulta (via tools existentes).

## O que o ktu tem hoje
`AGENTS.md` + skills; o `--root` confina o acesso a ficheiros.

## Pré-condições para reconsiderar
- Necessidade de consultar docs/código fora do workspace.
- Um modelo de acesso a caminhos externos compatível com o confinamento.

## Custo e risco
- **Esforço:** baixo (lista de referências no prompt + acesso read-only).
- **Risco de não fazer:** agente sem acesso a referência externa.
- **Risco de fazer:** acesso a caminhos **fora do root** — tem de ser explícito e
  read-only (guardrail).

## Decisão pendente
Promover só com necessidade de referência externa. Acesso **read-only**, explícito e
listado no prompt como **ponteiro** (nunca conteúdo carregado).
