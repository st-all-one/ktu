# Virtual models (roteamento de modelo)

- **Estado:** candidata (do `pi`) — **avaliar**.
- **Origem:** `REF/pi/packages/coding-agent/docs/virtual-models.md`; `wiki/revisao-pi.md` §2.4.
- **Âncora:** `crates/ktu-provider/`, `crates/ktu-cli/src/cli.rs`.

## O que é
Um modelo **selecionável** (virtual) que escolhe um modelo **físico** por pedido, segundo
um roteador (por tarefa/custo/estado). O pi separa **selecção** (registada em
`model_change`) de **dispatch** (o modelo que responde, na mensagem do assistente).

## Porque interessa
O ktu tem **um** modelo por sessão. Um virtual model é **kernel-friendly** (dado + roteador
**puro**) e casa com `--model`/`--effort`. É o primo do "modelo por fase" (backlog 8) mas
**mais simples** — não exige máquina de fases.

## O que o ktu tem hoje
`--model` e `--effort` por sessão; `Flavor` de wire por provider.

## Pré-condições para reconsiderar
- ≥2 modelos com custos/capacidades diferentes em uso.
- Um roteador **puro** (mesma entrada → mesma saída) e log da selecção vs dispatch.

## Custo e risco
- **Esforço:** médio (registo + roteador + log + resume).
- **Risco de não fazer:** baixo com 1–2 modelos.
- **Risco de fazer:** roteamento não-determinístico se o roteador não for puro.

## Decisão pendente
Promover quando houver ≥2 modelos. O roteador é uma função pura registada como dado;
a selecção e o dispatch ficam separados no log (para replay).
