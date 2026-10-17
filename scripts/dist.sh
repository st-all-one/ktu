#!/usr/bin/env bash
# Empacota o binário `ktu` num tarball versionado + checksum SHA-256.
#
# Determinístico: o alvo é `KTU_TARGET` (ou o host) e a versão vem do
# `Cargo.toml` do workspace. Saída em `target/dist/`:
#   ktu-<versão>-<triple>.tar.gz         (ktu + LICENSE + README.md)
#   ktu-<versão>-<triple>.tar.gz.sha256  (<hash>  <ficheiro>)
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"

triple="${KTU_TARGET:-$(rustc -vV | sed -n 's/^host: //p')}"
version="$(grep -m1 '^version = ' Cargo.toml | sed -E 's/^version = "(.*)"/\1/')"
out="$root/target/dist"
name="ktu-${version}-${triple}"
stage="$out/$name"

if [ -z "$triple" ] || [ -z "$version" ]; then
    echo "dist: não foi possível determinar o alvo/versão" >&2
    exit 1
fi

echo "dist: ktu ${version} → ${triple}" >&2
cargo build --release -p ktu-cli --locked

rm -rf "$stage"
mkdir -p "$stage"
cp target/release/ktu "$stage/ktu"
cp LICENSE README.md "$stage/"
tar -C "$out" -czf "$out/${name}.tar.gz" "$name"

if command -v sha256sum >/dev/null 2>&1; then
    ( cd "$out" && sha256sum "${name}.tar.gz" > "${name}.tar.gz.sha256" )
else
    ( cd "$out" && shasum -a 256 "${name}.tar.gz" > "${name}.tar.gz.sha256" )
fi

rm -rf "$stage"
echo "$out/${name}.tar.gz" >&2
