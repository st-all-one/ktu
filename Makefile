# ktu — alvos de qualidade e build.
# Portão único: `make check` (fmt --check + clippy -D warnings + test + firewall
# + governança (adr/surface) + links + doc + tamanho).
# O `crates/knudge` é workspace próprio (excluído da raiz): o portão dele é
# `make check-knudge`, chamado pela CI num passo à parte.

CARGO ?= cargo

.PHONY: check fmt clippy test build dist bench layers adr surface links doc check-knudge gates file-length quality deny machete typos supply-chain clean

## Portão completo local: formatação, lints, testes, firewall de camadas, links, doc e tamanho.
check: fmt clippy test layers adr surface links doc file-length

## Verifica formatação sem alterar.
fmt:
	$(CARGO) fmt --all -- --check

## Lints de todos os alvos com warnings como erro (I7).
clippy:
	$(CARGO) clippy --workspace --all-targets -- -D warnings

## Testes do workspace (inclui doc-tests).
test:
	$(CARGO) test --workspace

## Build de debug do workspace.
build:
	$(CARGO) build --workspace

## Empacota o binário `ktu` (tarball + checksum SHA-256) em `target/dist/`.
dist:
	./scripts/dist.sh

## Benchmark determinístico do núcleo → `wiki/benchmarks/ktu-bench.json`.
bench:
	$(CARGO) bench -p ktu-core --bench core

## Firewall de dependências entre camadas (§2.1).
layers:
	$(CARGO) run -q -p xtask -- check-layers

## Registos de governança: ADRs com `Alternatives considered` + postmortems (H11).
adr:
	$(CARGO) run -q -p xtask -- check-adr

## Teto de superfície pública: crescer exige subir `surface.toml` (H11).
surface:
	$(CARGO) run -q -p xtask -- check-surface

## Links internos da wiki (`wiki/**`, `README.md`, `STAGING.md`, `plan.md`) — L3/PM-0003.
links:
	$(CARGO) run -q -p xtask -- check-links

## Documentação sem avisos: links intra-doc do Rust (`broken_intra_doc_links` é `deny`) — L3.
## Medido: ~4 s com o alvo quente (o `test` já construiu as rlibs); ~34 s com alvo próprio vazio.
doc:
	RUSTDOCFLAGS="-D warnings" $(CARGO) doc --workspace --no-deps

## Portão do knudge (workspace próprio, `exclude` na raiz) — L1. A CI chama-o num passo à parte.
check-knudge:
	cd crates/knudge && $(MAKE) check

## Gates da Fase 2 (G10–G15): correm o `ktu` e arquivam artefactos em `wiki/gates/`.
## Não entram no `make check` (escrevem evidência versionada); correm-se no fecho de fase.
gates:
	$(CARGO) run -q -p xtask -- gates $(ARGS)

## Nenhum ficheiro de produção passa de 400 linhas.
file-length:
	./scripts/check_file_length.sh

## Bateria de qualidade p1–p4 com o ktu isolado (ver scripts/quality.sh).
quality:
	./scripts/quality.sh $(ARGS)

## Supply chain: advisories, licenças e wildcards (ver deny.toml).
deny:
	cargo deny check

## Bateria de supply chain completa (deny + machete + typos).
supply-chain: deny machete typos

## Dependências declaradas mas não usadas.
machete:
	cargo machete

## Erros ortográficos em código/docs (crate-ci/typos; vocabulário em typos.toml).
typos:
	typos

clean:
	$(CARGO) clean
