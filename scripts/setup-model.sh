#!/usr/bin/env bash
# Prepara o modelo de referência (Ornith-1.5-9B-GGUF) e o llama.cpp stock — CPU-only.
# Idempotente: só descarrega/constrói o que falta.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
MODELS="${KTU_MODELS_DIR:-$ROOT/models}"
LLAMA_DIR="${KTU_LLAMA_DIR:-$ROOT/.ktu/llama.cpp}"
LLAMA_REPO="${KTU_LLAMA_REPO:-https://github.com/ggerganov/llama.cpp}"
MODEL_REPO="ornith-ai/Ornith-1.5-9B-GGUF"
QUANT="${KTU_QUANT:-Q4_K_M}"
MODEL_FILE="Ornith-1.5-9B-${QUANT}.gguf"

mkdir -p "$MODELS"

echo "== modelo ($QUANT) =="
if [ -f "$MODELS/$MODEL_FILE" ]; then
  echo "ja existe: $MODELS/$MODEL_FILE"
elif command -v hf >/dev/null 2>&1; then
  hf download "$MODEL_REPO" "Ornith-1.5-9B-${QUANT}.gguf" --local-dir "$MODELS"
elif command -v huggingface-cli >/dev/null 2>&1; then
  huggingface-cli download "$MODEL_REPO" "Ornith-1.5-9B-${QUANT}.gguf" --local-dir "$MODELS"
else
  echo "erro: instale o Hugging Face CLI (pip install -U 'huggingface_hub[cli]')" >&2
  exit 1
fi

echo "== llama.cpp (stock) =="
if [ ! -d "$LLAMA_DIR/.git" ]; then
  git clone --depth 1 "$LLAMA_REPO" "$LLAMA_DIR"
fi

if [ ! -x "$LLAMA_DIR/build/bin/llama-server" ]; then
  cmake -S "$LLAMA_DIR" -B "$LLAMA_DIR/build" -DCMAKE_BUILD_TYPE=Release -DLLAMA_CURL=OFF
  cmake --build "$LLAMA_DIR/build" --config Release -j"$(nproc)" --target llama-server
fi

echo
echo "modelo:   $MODELS/$MODEL_FILE"
echo "servidor: $LLAMA_DIR/build/bin/llama-server"
echo "arranque: ./scripts/run-server.sh"
