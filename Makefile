# ktu — alvos de qualidade e build.
# Portão único: `make check` (fmt --check + clippy -D warnings + test + firewall + tamanho).

CARGO ?= cargo

.PHONY: check fmt clippy test build layers file-length quality clean

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

## Firewall de dependências entre camadas (§2.1).
layers:
	$(CARGO) run -q -p xtask -- check-layers

## Nenhum ficheiro de produção passa de 300 linhas.
file-length:
	./scripts/check_file_length.sh

## Bateria de qualidade p1–p4 com o ktu isolado (ver scripts/quality.sh).
quality:
	./scripts/quality.sh $(ARGS)

clean:
	$(CARGO) clean
