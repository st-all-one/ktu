# Fase 3 — notas de execução (Rust × princípios)

> Transversais: [NOTAS-RUST da Fase 2](../implementation_phase2/NOTAS-RUST.md) (N1–N8) continuam a
> valer. Aqui ficam só os **deltas** que a Fase 3 acrescenta.

- **N9 — sinais sem `unsafe`.** Ctrl-C não se apanha com `unsafe`/`libc` directo: usar o
  `crossterm` (já dependência do `tui`) ou um handler da borda que chama o `CancelToken` que já
  existe. `forbid(unsafe_code)` é global.
- **N10 — *raw mode* é gated.** Só quando `stdin`/`stderr` são TTY (`IsTerminal`); o caminho *pipe*
  é o que os gates usam (G12) e **não pode** mudar de comportamento.
- **N11 — PTY nos testes.** Testar interacção a sério exige um pseudo-terminal; quando for
  preciso, usar o utilitário `script` (util-linux) ou alimentar o canal de input do REPL
  directamente (a thread de leitura é injectável) — nunca `unwrap` num *spawn*.
- **N12 — idempotência por id.** A dedup é **por id**, nunca por texto: dois ids com o mesmo texto
  são duas mensagens (é o teste negativo do F1).
- **N13 — migração separada da leitura.** `migrate` (escrita) e `read_migrate` (leitura) são
  funções **distintas**; a segunda nunca remove chaves (lição do `goose`).
- **N14 — links validados por ficheiro.** `check-links` valida o **ficheiro** alvo; âncoras `#…`
  são aviso, não erro (falsos positivos matam um travão).
- **N15 — aditivo vs substitutivo.** A precedência global/local é **por tipo** e está declarada:
  guardrails **somam**, plugins **somam**, skills/receitas **substituem por nome**. Escrever a
  regra no `--help` e testá-la nos dois sentidos.
