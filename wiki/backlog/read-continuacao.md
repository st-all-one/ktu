# Continuação de leitura truncada

- **Estado:** **implementado** (H5; `wiki/implementation_phase2/H5-fiabilidade-loop.md`).
- **Origem:** `REF/katu/wiki/crates/DOSSIE_PENDENCIAS.md` §4; `wiki/revisao-katu.md` §3.12.
- **Âncora:** `crates/ktu-tools/src/fs_tools.rs`.

## O que é
O aviso de truncagem indica a **continuação** (`read <id>@<linha>`) em vez de só dizer "truncado"; a
leitura inexistente devolve `read.missing` legível (não `Unavailable`). Sem isto, o modelo repetia a
leitura até o guard de loop cortar.

## Porque interessa
Um ficheiro grande é lido truncado; sem o ponteiro de continuação, o modelo repete a leitura
idêntica (estagnação). Um `read.missing` legível distingue "não existe" de "falha transitória".

## O que o ktu tem hoje
`continu` = 14 hits (outros usos); a leitura truncada/inexistente devolve um erro genérico.

## Pré-condições para reconsiderar
- Leitura de ficheiros grandes no fluxo real.

## Custo e risco
- **Esforço:** baixo (mensagem com `@linha` + `read.missing`).
- **Risco de não fazer:** leituras repetidas → estagnação.
- **Risco de fazer:** ponteiro errado — testar com ficheiro truncado.

## Decisão pendente
Promover. Aviso de truncagem com continuação; `read.missing` legível; teste de ficheiro grande e
de ficheiro inexistente.
