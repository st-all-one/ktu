---
id: fact_01uh5g3z
type: fact
statement: "Packaging e releases: install.sh (fonte/release+checksum), dist.sh, release.yml e jobs CI msrv+dist"
created_at: 2026-10-17T11:24:37.836Z
body_hash: ea45025b
schema_version: 2
anchors: [scripts/dist.sh, install.sh]
---
scripts/dist.sh empacota target/release/ktu em target/dist/ktu-<versao>-<triple>.tar.gz + .sha256 (alvo KTU_TARGET default host, versao do Cargo.toml). install.sh descarrega a release latest (API GitHub), verifica SHA-256 e instala em ~/.local/bin, ou compila com KTU_FROM_SOURCE=1; KTU_RELEASE_BASE permite teste/auto-hospedagem. Makefile ganhou make dist. .github/workflows/release.yml dispara em tags v* (matriz x86_64-unknown-linux-gnu + aarch64-apple-darwin). ci.yml ganhou jobs msrv (cargo check --workspace --all-targets --locked em 1.99) e dist. Validado localmente em 4 caminhos; make check e make supply-chain verdes.