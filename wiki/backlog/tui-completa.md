# TUI completa

- **Estado:** deferido.
- **Origem:** katu `11-tui-e-ux.md`; `plan.md` §0 ("TUI extravagante").
- **Âncora:** `crates/ktu-cli/src/tui.rs`.

## O que é
Uma TUI rica (painéis, diffs, navegação, comandos) sobre a porta única — em vez da TUI
mínima actual.

## O que o ktu tem hoje
- TUI **mínima** (ratatui 0.30): cliente da porta, `run(server, session)`, drop-guard que
  restaura o terminal, `describe`/`truncate` testados, smoke sob PTY.
- Web mínima (`fetch` + SSE) como outro cliente.

## Porque não entrou
É "extravagante" por decisão (§0). A TUI mínima prova o desacoplamento (mesma sessão pela
mesma porta) — que é o objectivo; a riqueza de UX não serve a métrica do gate.

## Pré-condições para reconsiderar
- Uso interactivo como alvo principal (não a bateria/gate).
- Componentes concretos pedidos por uso real.

## Custo e risco
- **Esforço:** médio-alto (widgets, keybindings, estado).
- **Risco de não fazer:** UX austera.
- **Risco de fazer (YAGNI):** tempo gasto em UI antes do necessário (erro do katu).

## Decisão pendente
Adiar. A porta única já permite construir TUI completa sem tocar no kernel — o custo de
entrar depois é baixo.
