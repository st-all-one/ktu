# Confiança estatística (FDR / conformal)

- **Estado:** deferido.
- **Origem:** katu `confidence/`, `wiki/_ref/plan/`.
- **Âncora:** — (sem equivalente no ktu).

## O que é
Um sistema de **confiança calibrada** sobre respostas/decisões do agente, com controlo
de taxa de falsos positivos (FDR) e/ou predição conformal, para decidir quando confiar,
escalar ou abster.

## O que o ktu tem hoje
Nada. A confiança é implícita (o modelo responde ou chama tools); a verificação é
estrutural (guardrails, invariantes, testes).

## Porque não entrou
Não há consumidor: nenhuma decisão do loop hoje depende de um score de confiança.
Introduzi-lo exigiria dados rotulados e um caso de uso que o justifique.

## Pré-condições para reconsiderar
- Um ponto de decisão real que beneficie de abstenção/escalada por confiança
  (ex.: auto-aprovação de escrita, ou roteamento automático de modelo).
- Dados de avaliação suficientes para calibrar.

## Custo e risco
- **Esforço:** alto (estatística + calibração + dados).
- **Risco de não fazer:** baixo.
- **Risco de fazer (YAGNI):** complexidade estatística sem métrica que a valide.

## Decisão pendente
Manter fora até existir um ponto de decisão e dados. Provavelmente a última a entrar.
