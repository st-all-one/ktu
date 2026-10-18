---
id: decision_005aefug
type: decision
statement: H11: o teto de superficie passa a mecanico (check-surface no gate) e a medida e feita pelo binario
created_at: 2026-10-18T03:56:52.935Z
body_hash: 3ad12f38
schema_version: 2
revision: 3
anchors: [surface.toml, crates/ktu-cli/src/surface.rs, xtask/src/surface.rs, wiki/adr/0011-limite-de-400-linhas-e-firewall-mecanico.md]
---
O plano da H11 (wiki/implementation_phase2/H11-governanca.md) dizia que o teto de superficie era "conferido em revisao". Passou a ser mecanico: xtask check-surface le surface.toml e compara com a medida do proprio binario (ktu self surface --json), correndo no make check. Razao: um numero que so existe para ser olhado nao trava nada - o katu cresceu 70k linhas sem ninguem decidir crescer, e o ktu cresceu de 24 para 47 subcomandos em duas fases. A medida e feita pelo binario (arvore do clap + catalogo builtin + as tools do loop) e nao por regex no cli.rs, porque a segunda mede a forma do codigo e a primeira mede o que o utilizador ve. Contrapartida assumida: uma tool nova acrescentada pelo loop (fora do Toolbox) tem de ser contada explicitamente em crates/ktu-cli/src/surface.rs, e uma flag nova em qualquer subcomando obriga a subir o teto em surface.toml na mesma alteracao. Alternativas descartadas: (a) manter "conferido em revisao" como dizia o plano - nao trava nada e nao deixa rasto; (b) contar por regex sobre o cli.rs - mede a forma do codigo e nao o que o utilizador ve, e quebra ao primeiro refactor; (c) medir a partir de target/debug/ktu - pode ser um binario stale e medir a superficie de ontem; (d) alvo unico check-docs para ADRs e superficie - mistura registos de governanca com uma medida de produto. O alvo do plano chamava-se check-docs; chama-se check-adr (nome do backlog) e valida ADRs e postmortems com o mesmo codigo.