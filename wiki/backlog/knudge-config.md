# knudge — configuração em dois níveis (global/projeto)

- **Estado:** candidata (do `knudge` 0.6.1) — **avaliar** (consolida com [76](./config-precedencia.md)).
- **Origem:** `crates/knudge/wiki/integration/11_git_e_config.md`,
  `crates/knudge/wiki/specs/configuracao.md`; `wiki/revisao-knudge.md` §6.2.
- **Âncora:** `crates/ktu-cli/src/init.rs` (config, se adotada).

## O que é
Config em **dois níveis**: global (`$XDG_CONFIG_HOME/local/knudge/config.toml`) + projeto
(`<raiz>/<layout>/config.toml`), com **precedência projeto > global**, chave a chave. O `onboard`
clona o global **literalmente** (não segue mudanças futuras sem `--force`); **segredos só no
global** (o projeto é sanitizado em `Config::effective`); codec TOML próprio (leitura preserva
ordem, escrita canónica).

## Porque interessa
O ktu não tem config em ficheiro. O modelo do knudge é o mesmo que a página
[76 — config-precedencia](./config-precedencia.md) propõe, mas **já implementado e testado** —
incluindo a separação **configuração vs segredos** e a cópia literal no `init`.

## O que o ktu tem hoje
Defaults no código + flags de CLI; sem TOML.

## Pré-condições para reconsiderar
- Adoptar config em ficheiro (a página 76 é a decisão).

## Custo e risco
- **Esforço:** baixo-médio (adoptar o modelo do knudge).
- **Risco de não fazer:** defaults presos no código.
- **Risco de fazer:** segredos no projeto — o knudge já os exclui; seguir o mesmo guard.

## Decisão pendente
Consolidar em [76](./config-precedencia.md): adoptar o **modelo** do knudge (dois níveis,
precedência, segredos só no global, cópia literal no `init`). Não duplicar a decisão.
