# App desktop / UI

- **Estado:** candidata (do `goose`) — **não trazer**.
- **Origem:** `REF/goose/ui`, `documentation/docs/desktop-navigation`; `wiki/revisao-goose.md` §3.
- **Âncora:** `crates/ktu-cli/src/tui.rs`, `crates/ktu-cli/src/serve.rs` (web).

## O que é
Uma **aplicação desktop nativa** (macOS/Linux/Windows) como interface principal, além do
CLI/TUI.

## Porque não entra
O ktu é um kernel mínimo com CLI + TUI mínima + web mínima. Um app desktop é uma segunda
base de código (UI, packaging, updates) sem relação com a tese.

## Pré-condições para reconsiderar
- Uso não-técnico como alvo principal.
- A porta única (G6) já permite construir qualquer front-end **sem tocar no kernel** — o
  custo de entrar depois é baixo.

## Custo e risco
- **Esforço:** alto (UI + distribuição por SO).
- **Risco de não fazer:** UX austera.
- **Risco de fazer (YAGNI):** esforço de UI antes do necessário (erro do katu).

## Decisão pendente
Adiar indefinidamente; a porta é o contrato, a UI é substituível.
