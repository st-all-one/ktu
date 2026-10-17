# Visão / imagens

- **Estado:** candidata (do `pi`) — **avaliar**.
- **Origem:** `REF/pi/packages/coding-agent/src/utils/image-*.ts`,
  `core/utils/tool-result-images.ts`, `docs/message-types.md` (`ImageContent`);
  `wiki/revisao-pi-profunda.md` §1.9.
- **Âncora:** `crates/ktu-core/src/domain.rs`, `crates/ktu-provider/src/wire.rs`.

## O que é
Mensagens com `ImageContent` (base64 + mime); o pi trata redimensionamento, conversão,
orientação EXIF, imagem da área de transferência e imagens devolvidas por tools.

## Porque interessa
O ktu só lida com texto. Aceitar imagens (colar/anexar) e devolver imagens de tools é usabilidade
objectiva — mas exige um caminho de **bytes** no wire e no log.

## O que o ktu tem hoje
Apenas texto.

## Pré-condições para reconsiderar
- Um modelo/provider em uso que aceite imagens.

## Custo e risco
- **Esforço:** médio (content blocks + wire + log).
- **Risco de não fazer:** sem visão.
- **Risco de fazer:** log enorme — guardar **referência** (ficheiro) e não os bytes; redimensionar
  antes de enviar.

## Decisão pendente
Avaliar. Só com um modelo que aceite imagens; no log, referência (não bytes); redimensionar antes
de enviar; determinístico.
