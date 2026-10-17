# knudge — como consumir o `knudge-core` (dependência)

- **Estado:** candidata (do `knudge` 0.6.1) — **decidir**.
- **Origem:** `crates/knudge` (checkout local) · D214 (`knudge-core` publicável);
  `wiki/revisao-knudge.md` §5.
- **Âncora:** `Cargo.toml`, `crates/ktu-memory/Cargo.toml`.

## O que é
O `ktu-memory` depende de `knudge-core` por **path**. O checkout actual vive em `crates/knudge`
(local-only, como o `REF/`). Há três formas de o consumir de forma **reprodutível**:

| Via | Forma | Prós | Contras |
|---|---|---|---|
| **Path (actual)** | `{ path = "../../crates/knudge/crates/knudge-core" }` | simples; edita-se o motor | não reprodutível (checkout local) |
| **Git pinada** | `{ git = "https://github.com/st-all-one/knudge.git", tag = "v0.6.1" }` | reprodutível; sem vendor | exige rede no `cargo fetch`; `Cargo.lock` |
| **Submódulo** | `.gitmodules` + `git submodule add` | fonte versionada no repo | gesto de git; sincronização manual |
| **Publicada** | `knudge-core = "0.6"` (crates.io, quando publicado) | a mais simples | depende de publicação |

## Porque interessa
O projecto preza reprodutibilidade (supply-chain verde, `deny`/`machete`). Um path para um checkout
**local-only** quebra um clone novo. `knudge-core` é agora **publicável** (D214), logo a via
reprodutível existe.

## O que o ktu tem hoje
`knudge-core = { path = "../../crates/knudge/crates/knudge-core" }` (migração validada).

## Pré-condições para reconsiderar
- Decisão de versionamento (humana; o agente não commita).

## Custo e risco
- **Esforço:** baixo (uma linha no `Cargo.toml`).
- **Risco de não fazer:** clone novo não compila.
- **Risco de fazer:** pin desactualizado — bumpar conscientemente (semver + CHANGELOG do knudge).

## Decisão pendente
Decidir a via (recomendado: **git pinada por tag** enquanto não há publicação; ou **publicada**
quando existir). O `crates/knudge` fica como checkout de **exploração/desenvolvimento** do motor.
Registar a decisão (ADR).
