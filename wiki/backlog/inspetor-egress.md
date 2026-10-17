# Inspector de egress

- **Estado:** candidata (do `goose`) — **avaliar**.
- **Origem:** `REF/goose/crates/goose/src/security/egress_inspector.rs`;
  `wiki/revisao-goose-profunda.md` §1.13.
- **Âncora:** `crates/ktu-tools/src/bash.rs`, `crates/ktu-core/src/taint.rs`.

## O que é
Monitoriza/inspeta o tráfego de **saída** (exfiltração de dados) — a par do
`adversary_inspector` e do `scanner` — para apanhar o envio de segredos para serviços externos.

## Porque interessa
O ktu confina o `bash` e marca conteúdo não confiável (`taint.rs`); inspeccionar o que **sai**
(URLs, pipes para serviços externos) é a defesa complementar — o risco clássico de uma
injecção é exfiltrar `.env`/chaves.

## O que o ktu tem hoje
Confinamento do `bash` + `taint.rs`; sem inspecção de egress.

## Pré-condições para reconsiderar
- Evidência de tentativa de exfiltração (ou política de segurança a exigir).

## Custo e risco
- **Esforço:** médio (extrair hosts/comandos de rede e comparar com uma allowlist).
- **Risco de não fazer:** exfiltração passa.
- **Risco de fazer:** falsos positivos — allowlist de hosts explícita e erro acionável.

## Decisão pendente
Avaliar. Se entrar, **allowlist de hosts** como dado (não ML), erro acionável, determinístico;
consolidar com [deteccao-injecao-prompt](./deteccao-injecao-prompt.md).
