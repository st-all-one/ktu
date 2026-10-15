# ktu

Agente mínimo (2.ª tentativa). O núcleo é **puro**; todos os efeitos entram por
**portas** (`Clock`, `Rng`, `Fs`, `Env`, `Logger`, `Metrics`).

- **Memória do projecto:** knudge (`.knudge/`) — protocolo em [`AGENTS.md`](AGENTS.md).
- **Design:** [`plan.md`](plan.md). **Execução sequencial:** [`epic.md`](epic.md).
- **Portão único:** `make check` = `fmt` + `clippy -D warnings` + `test` +
  firewall de camadas (`xtask check-layers`) + tamanho de ficheiro (≤ 300 linhas).

## Estrutura

| Crate | Papel |
|---|---|
| `ktu-core` | núcleo puro: domínio, portas, loop agêntico |
| `ktu-tools` | thread de tools (fs, exec, memória) |
| `ktu-provider` | provider built-in: llama.cpp via `llama-server` |
| `ktu-port` | porta única (protocolo + transportes `Channel`/`Http`) |
| `ktu-memory` | porta `Memory` + motor knudge |
| `ktu-cli` | binário `ktu` (front-end embedded) |

## Uso

```bash
ktu init      # cria .agents/ (conhecimento) e .ktu/ (runtime)
make check    # portão de qualidade
```
