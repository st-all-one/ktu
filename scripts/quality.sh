#!/usr/bin/env bash
# Bateria de qualidade p1–p4 com o ktu como sujeito isolado (F5/G4).
# Constrói o binário `ktu` e delega a orquestração ao `xtask quality`.
#
# Uso:
#   ./scripts/quality.sh --dry-run                 # só mostra o plano
#   ./scripts/quality.sh --provider mock           # valida a mecânica sem modelo
#   KTU_SKIP_BUILD=1 ./scripts/quality.sh --only p1
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

PROFILE="${KTU_PROFILE:-debug}"
KTU_BIN="$ROOT/target/$PROFILE/ktu"
# URL do llama-server: KTU_BASE_URL (ou porta em KTU_PORT). Permite conviver
# com outro servidor já a ocupar o 8080 (ex.: katu-dev).
BASE_URL="${KTU_BASE_URL:-http://127.0.0.1:${KTU_PORT:-43111}}"

if [ "${KTU_SKIP_BUILD:-0}" != "1" ]; then
  if [ "$PROFILE" = "release" ]; then
    cargo build --release -p ktu-cli
  else
    cargo build -p ktu-cli
  fi
fi

exec cargo run -q -p xtask -- quality --ktu "$KTU_BIN" --base-url "$BASE_URL" "$@"
