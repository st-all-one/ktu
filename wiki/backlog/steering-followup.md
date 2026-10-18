# Steering / follow-up messages

- **Estado:** **implementado** (H6+) — steering entregue na fronteira segura, logado (`Event::Steer`) e exposto na porta (`Host::steer`); o REPL lê stdin numa thread dedicada e drena sem bloquear entre eventos.
- **Origem:** `REF/pi/packages/coding-agent/docs/how-pi-works.md`; `wiki/revisao-pi.md` §2.3.
- **Âncora:** `crates/ktu-port/src/protocol.rs`, `crates/ktu-core/src/agent/mod.rs`.

## O que é
O utilizador pode enviar mensagens **durante** um turno (*steering* — entram após o passo
do assistente) ou **após** o trabalho pendente (*follow-up*). Abortar devolve as mensagens
enfileiradas ao editor.

## Porque interessa
O ktu tem `cancel`/`continue` **fora-de-banda** na porta, mas não **enfileira** mensagens.
Corrigir o rumo sem matar o turno é usabilidade objectiva.

## O que o ktu tem hoje
Porta com `cancel`/`SetMode`/`continue`; o turno corre até ao fim ou é cancelado.

## Pré-condições para reconsiderar
- Uso interactivo onde corrigir a meio é frequente.
- Regra de injecção **determinística** (onde a mensagem entra) e logada.

## Custo e risco
- **Esforço:** médio (fila na porta + pontos de injecção no loop).
- **Risco de não fazer:** baixo (cancelar+repetir serve).
- **Risco de fazer:** não-determinismo no turno — exige ordem estável e evento no log.

## Decisão pendente
Promover se a interacção mostrar fricção. A injecção tem de ser **um ponto** no loop,
logado, com ordem determinística.
