# PTY persistente

- **Estado:** candidata (do `opencode`) — **avaliar**.
- **Origem:** `REF/opencode/packages/core/src/persistent-pty`; `wiki/revisao-opencode.md` §2.6.
- **Âncora:** `crates/ktu-tools/src/bash.rs`.

## O que é
Um **shell persistente** por sessão: `cd`, variáveis de ambiente e estado sobrevivem entre
comandos.

## Porque interessa
O ktu corre `bash` num **processo por comando** (cwd = root). Um PTY persistente melhora
usabilidade (navegar, activar ambientes) — mas **complica o confinamento**.

## O que o ktu tem hoje
`bash` com cwd = root, timeout/cap, análise de `argv` e confinamento de caminhos
(`analyze_paths`).

## Pré-condições para reconsiderar
- Necessidade real de estado entre comandos (ex.: venv, `cd`).
- Um modelo de confinamento compatível com um shell vivo.

## Custo e risco
- **Esforço:** médio (PTY + ciclo de vida + isolamento).
- **Risco de não fazer:** comandos mais verbosos (`cd x && ...`).
- **Risco de fazer:** **escapar ao confinamento** do `bash` (o `cd` persiste e contorna a
  análise de caminhos) — risco de segurança.

## Decisão pendente
Adiar. Se entrar, o confinamento tem de ser reavaliado (o `cd` persistente muda o modelo
de segurança); provavelmente fica atrás de opt-in e com root fixo.
