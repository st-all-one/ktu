# Allowlist (de plugins/comandos)

- **Estado:** candidata (do `goose`) — **avaliar**.
- **Origem:** `REF/goose/documentation/docs/guides/allowlist.md`;
  `wiki/revisao-goose-profunda.md` §1.12.
- **Âncora:** `crates/ktu-core/src/plugin.rs`, `crates/ktu-cli/src/plugins_cmd.rs`.

## O que é
Uma allowlist YAML (`id` + `command`) buscada de um URL (`GOOSE_ALLOWLIST`), cacheada e
refrescada no arranque; instalar uma extensão fora da lista é **recusado**.

## Porque interessa
O ktu instala plugins; uma allowlist (ou lista de comandos permitidos) é uma trave de
segurança simples para ambientes geridos — e é **dado** (lista), não código.

## O que o ktu tem hoje
Plugins com guardrails declarativos; sem allowlist de instalação.

## Pré-condições para reconsiderar
- Ambiente gerido/multi-utilizador.

## Custo e risco
- **Esforço:** baixo (lista + verificação na instalação).
- **Risco de não fazer:** instalação de plugin não confiável.
- **Risco de fazer:** allowlist desactualizada bloqueia — erro explícito e caminho local
  (`--allowlist-file`) além do URL.

## Decisão pendente
Avaliar. Lista local (sem rede) primeiro; URL opcional; recusa explícita com a entrada em falta.
