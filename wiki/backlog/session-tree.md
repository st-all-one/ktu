# Árvore de sessão (branching / fork / clone)

- **Estado:** candidata (do `pi`) — **avaliar** (alto valor).
- **Origem:** `REF/pi/packages/coding-agent/docs/{how-pi-works,sessions,session-format}.md`;
  `wiki/revisao-pi.md` §2.1.
- **Âncora:** `crates/ktu-core/src/session/mod.rs`, `.ktu/sessions/*.jsonl`.

## O que é
O log da sessão é uma **árvore** (`id`/`parentId`), não uma linha; cada caminho é um
*branch*. `/tree` move dentro da sessão, `/fork` cria uma nova a partir de uma mensagem
anterior, `/clone` copia o branch activo. Ao sair de um branch, o pi pode **resumi-lo** e
anexar o resumo ao novo. O modelo vê só o **branch activo**.

## Porque interessa
É a diferença **mais estrutural** face ao ktu (log linear) e o katu já a tinha deferido
("S2 árvore de sessão"). É **kernel** (o log é a fonte da verdade) e usabilidade objectiva
(explorar alternativas sem perder trabalho). O resumo de branch reutiliza a compactação
aditiva existente.

## O que o ktu tem hoje
`SessionLog` **linear** com eventos; `resume` por id; compactação aditiva; checkpoint.

## Pré-condições para reconsiderar
- Necessidade real de explorar alternativas sem perder o caminho anterior.
- Desenho de `parent_id` nos eventos + projecção por branch + migração de logs lineares.

## Custo e risco
- **Esforço:** médio-alto (eventos + projecção + comandos + migração + testes de ouro).
- **Risco de não fazer:** baixo (o log linear funciona e passou o gate).
- **Risco de fazer:** tocar no **log (fonte da verdade)** — exige preservar I1–I3.

## Decisão pendente
Promover se houver uso de "ramos" (comparar abordagens). Fazer por **aditivo**: v1 linear
migra para v2 árvore; nunca quebrar logs existentes.
