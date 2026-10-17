# ktu — alvos de qualidade e build.
# Portão único: `make check` (fmt --check + clippy -D warnings + test + firewall + tamanho).

CARGO ?= cargo

.PHONY: check fmt clippy test build dist bench layers file-length quality deny machete typos supply-chain clean

## Portão completo local: formatação, lints, testes, firewall de camadas e tamanho de ficheiro.
check: fmt clippy test layers file-length

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

## Nenhum ficheiro de produção passa de 300 linhas.
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
