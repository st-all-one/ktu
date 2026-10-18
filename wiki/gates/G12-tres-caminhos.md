# G12 — Independência do caminho (M4)

- **Veredicto:** ✅ **verde** (4/4) · **Artefactos:** `artifacts/G12/` · **Data:** 2026-10-18
- **Critério:** [`GATES.md`](../implementation_phase2/GATES.md) §G12 — *«`derive_messages(log)`
  byte-idêntico nos três caminhos»*.

## Cenário

O **mesmo prompt** (`le a.txt`) por **três caminhos**, no mesmo projecto:

```bash
ktu run "le a.txt" --provider mock --session a                       # 1. efémero → sessão
printf 'le a.txt\n/quit\n' | ktu session b --provider mock          # 2. REPL
cargo run -q -p ktu-embed -- --root . --session c "le a.txt"          # 3. crate (fachada)
```

O terceiro caminho é o **crate de exemplo** (`examples/ktu-embed`), que incorpora o kernel pela
fachada `Ktu::builder()`; para que a comparação seja literal, ele usa **o mesmo** provider de
demonstração (`ktu_core::testing::EchoProvider`) e **o mesmo** host de plugins do CLI.

## Verificações

| # | Critério | Observado |
|---|---|---|
| 1 | os três caminhos terminam com sucesso | exit 0 · 0 · 0 |
| 2 | o log é **byte-idêntico** nos três | `fnv64: run=c21e1ff7afb821f3 repl=c21e1ff7afb821f3 crate=c21e1ff7afb821f3` |
| 3 | cada caminho fecha o turno (I5) | `turn_end` = 1 em cada log |
| 4 | **determinismo** (I6): o mesmo caminho duas vezes | `run` = repetido (`c21e1ff7afb821f3`) |

O log não tem *timestamps* (a hora vive no *checkpoint* e no log de diagnóstico, não no registo de
eventos) — é isso que torna a igualdade possível e a prova forte: não é «equivalência de grau», é
**byte-a-byte**.

## Artefactos

`run.jsonl`, `repl.jsonl`, `crate.jsonl` — os três logs (o `diff` entre eles é vazio).

## Notas

- O caminho do **REPL** partilha o log com o `run` também por teste em `make check`
  (`run_and_repl_produce_the_same_log`, `crates/ktu-cli/tests/session.rs`).
- O caminho do **crate** obrigou a alinhar as portas: sem o mesmo provider e o mesmo host de
  plugins, o log diferia nos eventos `plugin_dispatch` (que **não** projectam mensagens, mas
  mostrariam uma composição diferente). Com eles, a igualdade é literal — e o exemplo passa a
  demonstrar a composição completa.
