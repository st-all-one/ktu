# Loop de feedback

- **Estado:** deferido.
- **Origem:** katu `feedback.rs`.
- **Âncora:** — (sem equivalente no ktu).

## O que é
Um mecanismo de **feedback explícito** do utilizador (ou de um verificador) sobre o
resultado de um turno, que realimenta a memória/prompt (ex.: "esta resposta foi boa/má",
"preferir X a Y"), fechando o ciclo de melhoria.

## O que o ktu tem hoje
- `kd` (knudge) para conhecimento curado manualmente.
- A bateria `quality_test` (avaliação offline por equivalência).

## Porque não entrou
Não há canal de feedback em runtime, nem consumidor (nenhuma decisão do loop usa
preferências). A memória é curada explicitamente.

## Pré-condições para reconsiderar
- Um front-end que recolha feedback (TUI/web) e uma decisão do loop que o consuma.
- Um formato estável de preferência (evitar "texto livre como contrato").

## Custo e risco
- **Esforço:** médio.
- **Risco de não fazer:** baixo.
- **Risco de fazer (YAGNI):** recolher sinal que ninguém usa.

## Decisão pendente
Adiar até haver UI de feedback **e** um consumidor no loop.
