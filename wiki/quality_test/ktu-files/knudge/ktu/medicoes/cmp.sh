#!/usr/bin/env bash
# Compara duas saidas de sonda (rust vs node) CAMPO A CAMPO, forçando comparacao TEXTUAL.
# Uso: bash cmp.sh out_rs.txt out_node.txt
# A sentinela "@" evita o strnum do awk, que faria "1e+21" == "1000000000000000000000".
set -u
RS=${1:-out_rs.txt}
NODE=${2:-out_node.txt}
paste "$RS" "$NODE" > cmp.txt
awk -F'\t' '
{
  n = split($1, a, " ");
  m = split($2, b, " ");
  kind = a[1];
  v1 = a[n] "@";
  v2 = b[m] "@";
  if (v1 == v2) { ok++ }
  else { bad++; printf "DIVERGE  %-8s rust=%-26s node=%-26s\n", kind, a[n], b[m] }
}
END { printf "\nprobes=%d  texto-identicos=%d  divergentes=%d\n", ok+bad, ok, bad }
' cmp.txt
echo "(probes numericos com 16 digitos hex = bits IEEE-754; probes 'fmt' = numero->texto)"
