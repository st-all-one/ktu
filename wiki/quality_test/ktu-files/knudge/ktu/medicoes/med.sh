#!/usr/bin/env bash
# Mede o piso de arranque de processo (mediana de N execucoes) para cada runtime.
set -u
N=${N:-15}
OUT=.scratch/out.txt

med() {
  local label="$1"; shift
  local -a ms=()
  local i s e d
  for ((i=0; i<N; i++)); do
    s=$(date +%s%N)
    "$@" > "$OUT" 2>&1
    e=$(date +%s%N)
    d=$((e-s))
    d=$((d/1000))
    ms+=("$d")
  done
  printf '%s\n' "${ms[@]}" | sort -n | awk -v n="$N" -v l="$label" '{a[NR]=$1} END{printf "%-24s min=%6d  mediana=%6d  p95=%6d  max=%6d  (us)\n", l, a[1], a[int(n/2)+1], a[int(n*0.95)+1], a[n]}'
}

med "kd --help"              kd --help
med "kd self version"        kd self version
med "node -e ''"             node -e ""
med "bun -e ''"              bun -e ""
med "deno eval ''"           deno eval "" --no-config
med "node fs+crypto+path"    node -e "require('node:fs');require('node:crypto');require('node:path')"
med "node fs+crypto+zlib"    node -e "require('node:fs');require('node:crypto');require('node:zlib');require('node:url');require('node:os')"
