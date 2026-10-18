# 0011 — Limites mecânicos: 400 linhas, camadas, superfície

- **Status:** aceite
- **Supersedes:** —
- **Superseded by:** —
- **Âncora:** `scripts/check_file_length.sh`, `xtask/src/main.rs`, `surface.toml`

## Contexto

O katu cresceu até 70k linhas sem ninguém decidir crescer: não havia limite que obrigasse a
escolher. O ktu precisa de limites que **falhem sozinhos** — porque disciplina em revisão não
sobrevive a pressa.

## Decisão

Três limites **mecânicos**, todos no `make check`:

1. **Tamanho**: nenhum ficheiro de produção passa de **400 linhas** (`crates/*/src`,
   `examples/*/src`, `xtask/src`; 300 no workspace do knudge).
2. **Camadas**: o firewall de dependências é imposto por `xtask check-layers`, não por revisão.
3. **Superfície**: `surface.toml` tem o **teto** de subcomandos/flags/tools; `xtask check-surface`
   falha se a medida crescer sem subir o teto na mesma alteração (regra de poda M1: dizer que
   consumidor a superfície nova serve).

## Alternatives considered

| Alternativa | Porque não |
|---|---|
| Confiar em revisão de código | não escala e não deixa rasto; o katu é o contra-exemplo |
| 300 linhas em todo o lado | o `ktu-cli` tem despacho por comando e ficaria cheio de módulos triviais; 400 é o ponto onde o ficheiro ainda se lê |
| Limite só nos crates do núcleo | foi nas bordas (CLI) que o crescimento apareceu primeiro |
| Teto de superfície por contagem estática (regex no `cli.rs`) | mede a *forma* do código, não o que o utilizador vê; a árvore do `clap` e o catálogo de tools são a medida honesta |
| Sem teto de superfície (só revisão) | a CLI cresceu de 24 para 47 subcomandos em duas fases — sem número não há travão |

## Consequências

- Um ficheiro que cresce obriga a uma decisão (partir o módulo), não a uma discussão.
- Subir o teto é um acto explícito e visível no diff (`surface.toml`).
- Custo: falsos positivos possíveis no limite de linhas (mitigado: `tests/` fora de `src` isentos);
  a medida da superfície é documentada em `crates/ktu-cli/src/surface.rs`.
