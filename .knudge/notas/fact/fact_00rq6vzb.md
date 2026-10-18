---
id: fact_00rq6vzb
type: fact
statement: H11: metodo de medicao com base de evidencia e teto de superficie mecanico (ktu self surface)
created_at: 2026-10-18T03:56:52.352Z
body_hash: d4f47456
schema_version: 2
anchors: [wiki/metodo-medicao.md, surface.toml, crates/ktu-cli/src/surface.rs, xtask/src/surface.rs]
---
H11 (task_01j0lf38). (a) wiki/metodo-medicao.md fixa a regra: todo o item de optimizacao exige formula + artefacto + teste que o trava, com base de evidencia declarada (measured/inferred/provider_reported/unpriced), A/B por mediana e adoptar-ou-reverter (>=20% no alvo OU remocao de complexidade; senao reverter e escrever a rejeicao com o numero). O artefacto do bench passa a schema 2 com basis por amostra e por bloco (memory/binary) e a regra entrou no plan.md 4.1; os numeros da leitura foram actualizados (projecao de 10k eventos ~0,5 ms, turno de 1 passo ~7 us). (b) Teto de superficie: surface.toml com [ceiling] 47 subcomandos/87 flags/13 tools, medidos pelo proprio binario em ktu self surface (arvore do clap: subcomandos visiveis e flags declaradas por comando, com as globais contadas uma vez; tools: catalogo builtin maximo do Toolbox + as tools do loop subagent/tool_search) e conferidos por xtask check-surface no make check (make surface). Crescer exige subir o teto na mesma alteracao - a regra de poda M1 obriga a dizer que consumidor a superficie nova serve. Decisao de desenho: a medida e feita pelo binario (cargo run -q -p ktu-cli -- self surface --json) e nao por regex sobre o cli.rs, porque mede o que o utilizador ve e nao a forma do codigo.