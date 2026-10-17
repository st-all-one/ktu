# `prime` byte-idêntico por versão

- **Estado:** candidata (do `katu`, ADR 0019) — **avaliar**.
- **Origem:** `REF/katu/wiki/_ref/adr/0019-superficie-cli-tui-v2.md`; `wiki/revisao-katu.md` §3.16.
- **Âncora:** `crates/ktu-core/src/prompt.rs`.

## O que é
O `prime` (instrução de sistema inicial) é **estático e byte-idêntico por versão**; um `prime` por
grupo. Versionar o prime torna-o um contrato estável e comparável.

## Porque interessa
O ktu compõe o prompt a cada turno. Um prime versionado (a) estabiliza a cache de prefixo,
(b) permite comparar versões, (c) torna o prompt auditável.

## O que o ktu tem hoje
`prompt.rs` compõe o sistema; sem versão do prime.

## Pré-condições para reconsiderar
- Um consumidor do prime (CLI/TUI/`serve`) que beneficie de o ver estável.

## Custo e risco
- **Esforço:** baixo (constante de versão + teste byte-a-byte).
- **Risco de não fazer:** prime a mudar sem rasto.
- **Risco de fazer:** rigidez — a versão sobe com a mudança, explicitamente.

## Decisão pendente
Avaliar. Se o prime for composto de partes variáveis (AGENTS.md, skills), versionar só a **parte
estática** e testá-la byte-a-byte.
