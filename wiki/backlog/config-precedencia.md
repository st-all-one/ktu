# Configuração global/local e `--params`

- **Estado:** candidata (do `katu`, ADR 0020) — **avaliar**.
- **Origem:** `REF/katu/wiki/_ref/adr/0020-configuracao-global-local.md`;
  `wiki/revisao-katu.md` §3.13.
- **Âncora:** `crates/ktu-cli/src/init.rs`, `crates/ktu-cli/src/cli.rs`.

## O que é
Uma config global por SO (`~/.config/.../ktu.toml`), override local `<projeto>/.ktu/ktu.toml`
(precedência **projeto > global**, chave a chave), **conjunto fechado de chaves**
(`deny_unknown_fields`), e `--params` (toda a config do comando) **exclusivo** com flags (ambos →
exit 2).

## Porque interessa
O ktu tem defaults no código e flags no CLI; falta um lar para os padrões do utilizador e um
override por projecto. `--params` simplifica scripts; o XOR evita ambiguidade.

## O que o ktu tem hoje
`config` = 17 hits (provavelmente `.agents/`); sem TOML de config nem precedência explícita.

## Pré-condições para reconsiderar
- Chaves a configurar (provider/modelo/effort/behavior).

## Custo e risco
- **Esforço:** médio (TOML + precedência + validação de chaves + XOR).
- **Risco de não fazer:** defaults presos no código.
- **Risco de fazer:** colisão escalar/tabela em TOML (o katu teve de a travar) — guard
  `keys_have_no_scalar_table_collisions`.

## Decisão pendente
Avaliar. Precedência projeto > global; chaves fechadas; `--params` XOR flags; guard de colisão
escalar/tabela; testes de precedência e de chave desconhecida.
