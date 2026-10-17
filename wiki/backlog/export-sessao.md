# Export de sessão (Markdown)

- **Estado:** candidata (do `goose`) — a avaliar.
- **Origem:** `REF/goose/crates/goose/src/session/export_markdown.rs`;
  `wiki/revisao-goose.md` §2.7.
- **Âncora:** `crates/ktu-cli/src/memory_cmd.rs`, `.ktu/sessions/*.jsonl`.

## O que é
Exportar a **sessão** (turnos, tools, decisões) para Markdown — para revisão, handoff ou
arquivo. O goose tem também HTML.

## Porque interessa
O ktu já tem `ktu memory export --md` (vista derivada da memória) e o log é a fonte da
verdade; exportar a sessão em Markdown é **barato** e útil para revisão humana.

## O que o ktu tem hoje
`ktu memory list/export`, `ktu doctor`; o log de sessão é JSONL.

## Pré-condições para reconsiderar
- Necessidade de partilhar/rever uma sessão fora do terminal.

## Custo e risco
- **Esforço:** baixo (render do log para Markdown).
- **Risco de não fazer:** baixo.
- **Risco de fazer (YAGNI):** mais um formato de output.

## Decisão pendente
Promover se houver uso de revisão/handoff; derivar sempre do **log** (nunca uma segunda
fonte da verdade).
