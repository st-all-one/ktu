#!/usr/bin/env bash
# Gate de tamanho de ficheiro: ficheiros de produção em crates/*/src, examples/*/src e
# xtask/src ≤ 400 linhas.
# Módulos de teste embutidos (#[cfg(test)]) contam; por isso o limite é aplicado ao ficheiro.
# `crates/knudge/` é um checkout de exploração (excluído do workspace) — não é código do ktu.
set -euo pipefail

limit=400
status=0

while IFS= read -r -d '' file; do
    lines=$(wc -l <"$file")
    if ((lines > limit)); then
        printf 'ERRO: %s tem %d linhas (limite %d)\n' "$file" "$lines" "$limit" >&2
        status=1
    fi
done < <(find crates examples xtask -type f -name '*.rs' -path '*/src/*' -not -path 'crates/knudge/*' -print0)

if ((status == 0)); then
    printf 'file-length: ok (limite %d)\n' "$limit"
fi

exit "$status"
