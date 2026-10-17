# Saída estruturada (`response_format` json_schema)

- **Estado:** candidata (do `katu`, ADR 0025) — **avaliar** (com ressalva).
- **Origem:** `REF/katu/wiki/_ref/adr/0025-decodificacao-estruturada-por-schema.md`;
  `wiki/revisao-katu.md` §3.6.
- **Âncora:** `crates/ktu-provider/src/wire.rs`.

## O que é
`response_format: {json_schema}` derivado das tools do pedido, **opt-in** e **fail-open** (400 ⇒
repetir sem o campo). Fecha a classe de erro `Decode` por construção. Custo: **+962 B**/pedido,
~4× a codificação.

## Porque interessa
Um JSON de tool call truncado mata o turno em `Decode` — nenhuma política o recupera. O schema
restringe a geração por construção. **Mas** o katu **reverteu-o para `off`**: com `oneOf` de tool
calls sem **variante terminal de texto**, o modelo local fica obrigado a emitir sempre uma tool call
e o turno corre até ao teto.

## O que o ktu tem hoje
`structured` = 0 hits.

## Pré-condições para reconsiderar
- Um schema **com variante terminal** de texto / "sem tool".
- A/B que garanta que não parte a terminação.

## Custo e risco
- **Esforço:** médio.
- **Risco de não fazer:** turnos perdidos em `Decode`.
- **Risco de fazer:** **parte a terminação** (o erro do katu) — só com variante terminal e opt-in.

## Decisão pendente
Avaliar. Opt-in, fail-open, **só** com variante terminal de texto; sem isso, não fazer (repetir o
erro do katu seria desperdício medido).
