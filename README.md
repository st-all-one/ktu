# ktu

Agente mínimo (2.ª tentativa). O núcleo é **puro**; todos os efeitos entram por
**portas** (`Clock`, `Rng`, `Fs`, `Env`, `Logger`, `Metrics`).

- **Memória do projecto:** knudge (`.knudge/`) — protocolo em [`AGENTS.md`](AGENTS.md).
- **Design:** [`plan.md`](plan.md). **Execução sequencial:** [`epic.md`](epic.md).
- **Portão único:** `make check` = `fmt` + `clippy -D warnings` + `test` +
  firewall de camadas (`xtask check-layers`) + tamanho de ficheiro (≤ 400 linhas).

## Estrutura

| Crate | Papel |
|---|---|
| `ktu-core` | núcleo puro: domínio, portas, loop agêntico |
| `ktu-tools` | thread de tools (fs, exec, memória) |
| `ktu-provider` | provider built-in: llama.cpp via `llama-server` |
| `ktu-port` | porta única (protocolo + transportes `Channel`/`Http`) |
| `ktu-memory` | porta `Memory` + motor knudge |
| `ktu-cli` | binário `ktu` (front-end embedded) |

## Instalação

```bash
# a partir de uma release (binário pré-compilado + checksum)
curl -fsSL https://raw.githubusercontent.com/st-all-one/ktu/main/install.sh | bash

# a partir do código-fonte
KTU_FROM_SOURCE=1 ./install.sh
```

O `install.sh` instala em `~/.local/bin` (configurável com `KTU_INSTALL_DIR`).
Variáveis: `KTU_VERSION` (default `latest`), `KTU_REPO`, `KTU_FROM_SOURCE`,
`KTU_SOURCE_DIR`, `KTU_RELEASE_BASE`.

Para empacotar localmente:

```bash
make dist   # → target/dist/ktu-<versão>-<triple>.tar.gz (+ .sha256)
```

## Uso

```bash
ktu init      # cria .agents/ (conhecimento) e .ktu/ (runtime)
ktu run "…"   # um turno; `ktu session` abre o REPL
make check    # portão de qualidade
```

**Embutir noutro projecto Rust:** a fachada `ktu_core::facade::Ktu` (uso #1) — construtor com
portas injectadas, `ktu.turn(&mut log, prompt)` e sessões pela porta `Fs`. Guia completo em
[`wiki/integration/`](wiki/integration/README.md); exemplo compilado:
[`examples/ktu-embed`](examples/ktu-embed/src/main.rs) (`cargo run -p ktu-embed -- --backend scripted "olá"`).

**Prova (gates da Fase 2):** `make gates` corre o binário de ponta a ponta nos seis gates G10–G15
(cadeia longa, multi-modelo, três caminhos, concorrência, project-bound, memória) e arquiva os
artefactos em [`wiki/gates/`](wiki/gates/README.md).

**Governança (processo, não feature):** as decisões com o que perderam vivem em
[`wiki/adr/`](wiki/adr/README.md); o que **falta** (e o que reabre cada item) em
[`wiki/pendencias.md`](wiki/pendencias.md); como se mede antes de optimizar em
[`wiki/metodo-medicao.md`](wiki/metodo-medicao.md); a superfície tem **teto** em
[`surface.toml`](surface.toml) (`ktu self surface`). Tudo conferido no `make check`
(`xtask check-adr` / `check-surface`).
