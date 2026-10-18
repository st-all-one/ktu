#!/usr/bin/env bash
# Arranca o llama-server (stock) com o modelo de referência Ornith-1.5-9B (CPU-only).
# Contexto, quant e caminhos configuráveis por variáveis de ambiente.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
QUANT="${KTU_QUANT:-Q4_K_M}"
MODEL="${1:-${KTU_MODEL_PATH:-$ROOT/models/Ornith-1.5-9B-${QUANT}.gguf}}"
SERVER="${KTU_LLAMA_SERVER:-$ROOT/.ktu/llama.cpp/build/bin/llama-server}"
CTX="${KTU_CTX:-32768}"
HOST="${KTU_HOST:-127.0.0.1}"
PORT="${KTU_PORT:-43111}"

if [ ! -f "$MODEL" ]; then
  echo "erro: modelo inexistente: $MODEL (corra ./scripts/setup-model.sh)" >&2
  exit 1
fi
if [ ! -x "$SERVER" ]; then
  echo "erro: llama-server inexistente: $SERVER (corra ./scripts/setup-model.sh)" >&2
  exit 1
fi

# As bibliotecas do llama.cpp vivem ao lado do binário: sem isto o carregador
# dinâmico falha (`libllama-server-impl.so: cannot open shared object file`).
export LD_LIBRARY_PATH="$(dirname "$SERVER")${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"

# -fa on + KV q8_0: poupa RAM no host CPU-only (17 GiB, sem GPU dedicada).
exec "$SERVER" \
  -m "$MODEL" \
  --host "$HOST" --port "$PORT" \
  -ngl 0 -fa on -c "$CTX" \
  --cache-type-k q8_0 --cache-type-v q8_0 \
  --jinja \
  --temp 1.0 --top-p 0.95 --top-k 20 --min-p 0.0 \
  --presence-penalty 1.5 --repeat-penalty 1.0
