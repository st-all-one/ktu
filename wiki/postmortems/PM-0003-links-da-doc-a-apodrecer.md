# PM-0003 — Links internos da wiki a apodrecer em silêncio

- **Data:** 2026-10-18 (H10) · **Onde:** `wiki/`

## Contexto

A wiki do ktu cresceu para dezenas de ficheiros com muitos links internos (`../backlog/x.md`,
`../implementation_phase2/…`). Nenhum gate verificava que os destinos existiam.

## Impacto

Havia **dois links quebrados** (`revisao-opencode.md`) que ninguém detectou durante fases — só
apareceram quando um script de verificação de links foi escrito à mão, durante a H10. Links
quebrados numa wiki de decisões custam caro: a decisão parece não ter fonte.

## Causa

O `make check` valida código (fmt, clippy, testes, camadas, tamanho) e a wiki está **fora** do
alcance dessas ferramentas; o `cargo doc` (que verifica links intra-doc do Rust) também não está no
portão. Documentação era, por construção, o único artefacto sem travão.

## Correção

Os dois links foram corrigidos e a wiki voltou a estar internamente consistente (verificado com um
script de resolução de links relativos).

## Prevenção

- Os registos de governança (`wiki/adr/`, `wiki/postmortems/`) passam a ter **gate próprio**
  (`xtask check-adr` no `make check`): id, secções obrigatórias, substituição coerente e índice sem
  deriva.
- A wiki **passou a ter** verificação de links no portão: `xtask check-links` (dentro do
  `make check`, alvo `links`) resolve os destinos relativos de `wiki/**` + `README.md` +
  `STAGING.md` + `plan.md`; o `cargo doc` com `RUSTDOCFLAGS="-D warnings"` cobre os links
  *intra-doc* do Rust. O critério de reabertura desta prevenção cumpriu-se na **F7** (gate G22) —
  ver [`G22`](../gates/G22-travoes.md).
- Regra de escrita: ao mover/renomear um ficheiro da wiki, procurar quem o cita
  (`grep -rn "<nome>" wiki/`) na mesma alteração.
