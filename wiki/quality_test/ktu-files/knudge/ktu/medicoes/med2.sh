#!/usr/bin/env bash
# Mede o data path (listar + ler + dividir frontmatter + contar chaves + somar bytes)
# sobre 1000 notas. Mediana de N execucoes.
set -u
N=${N:-11}
OUT=.scratch/bench_out.txt

med() {
  local label="$1"; shift
  local i s e d
  local -a ms=()
  for ((i=0; i<N; i++)); do
    s=$(date +%s%N)
    "$@" > "$OUT" 2>&1
    e=$(date +%s%N)
    d=$((e-s)); d=$((d/1000))
    ms+=("$d")
  done
  printf '%s\n' "${ms[@]}" | sort -n | awk -v n="$N" -v l="$label" '{a[NR]=$1} END{printf "%-22s min=%7d  mediana=%7d  p95=%7d  max=%7d  (us)\n", l, a[1], a[int(n/2)+1], a[int(n*0.95)+1], a[n]}'
}

med "rust read+parse 1000" ./.scratch/path_rs
med "node read+parse 1000" node .scratch/path.mjs
med "bun  read+parse 1000" bun .scratch/path.mjs
echo "--- saida de referencia (uma execucao):"
./.scratch/path_rs
node .scratch/path.mjs
