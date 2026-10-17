---
id: fact_01u0njn8
type: fact
statement: TUI ratatui (ktu tui) e pagina web sao clientes HttpPort; evento server.start
created_at: 2026-10-17T02:06:38.746Z
body_hash: 292528c8
schema_version: 2
anchors: [crates/ktu-cli/src/tui.rs]
---
crates/ktu-cli/src/tui.rs: TUI minima que liga a ktu serve por HttpPort (eventos em tempo real + prompt). A pagina web (INDEX_HTML) usa fetch+EventSource. Novo id de evento estavel server.start. axum 0.8 e ratatui 0.30 (sem paste).