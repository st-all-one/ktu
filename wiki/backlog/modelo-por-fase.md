# Modelo por passo/fase + controlo de thinking

- **Estado:** deferido (o *seam* existe; o consumidor não).
- **Origem:** katu `model.rs` + `StepModel`/`PhaseModel` + `Control::SetModel/Thinking`,
  ADR `0012`; `plan.md` §2.12-B.
- **Âncora:** `crates/ktu-cli/src/cli.rs` (`--model`, `--effort`), `crates/ktu-port`.

## O que é
Escolher **modelo e grau de pensamento por passo ou por fase** do trabalho, controlado
pelo utilizador em runtime, logado e sobrevivente a `resume` (ex.: `deep` a planear,
`fast` a verificar).

## O que o ktu tem hoje
- Modelo e esforço por sessão (`--model`, `--effort`), com `SetMode` out-of-band na porta.
- Um `Event::ModeSet` logado/persistente; **não** há `Event::PhaseTransition` nem
  `SetModel`/`SetThinking`.

## Porque não entrou
O mecanismo do katu estava testado mas **inerte** (nenhum evento de fase era emitido →
política por fase era no-op). No ktu, o loop não tem máquina de fases.

## Pré-condições para reconsiderar
- Uma **máquina de fases** no loop que emita transições (o pré-requisito real).
- Um ganho medido de custo/qualidade por roteamento.

## Custo e risco
- **Esforço:** médio (fases no loop + controlo na porta + persistência).
- **Risco de não fazer:** baixo.
- **Risco de fazer (YAGNI):** mecanismo sem emissor — exactamente o estado inerte do katu.

## Decisão pendente
Adiar. Só faz sentido **depois** de existir uma máquina de fases com consumidor.
