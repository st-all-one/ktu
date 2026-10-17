#!/usr/bin/env bash
# Instala o binário `ktu` a partir de uma release do GitHub (ou do código-fonte).
#
#   curl -fsSL https://raw.githubusercontent.com/st-all-one/ktu/main/install.sh | bash
#
# Variáveis:
#   KTU_VERSION      tag/versão a instalar (default: latest)
#   KTU_INSTALL_DIR  destino (default: ~/.local/bin)
#   KTU_FROM_SOURCE  1 = compila do checkout em vez de descarregar a release
#   KTU_SOURCE_DIR   raiz do checkout quando KTU_FROM_SOURCE=1 (default: $PWD)
#   KTU_REPO         owner/repo (default: st-all-one/ktu)
set -euo pipefail

repo="${KTU_REPO:-st-all-one/ktu}"
version="${KTU_VERSION:-latest}"
install_dir="${KTU_INSTALL_DIR:-$HOME/.local/bin}"

detect_triple() {
    local os arch
    case "$(uname -s)" in
        Linux)  os="unknown-linux-gnu" ;;
        Darwin) os="apple-darwin" ;;
        *) echo "install: SO não suportado: $(uname -s)" >&2; return 1 ;;
    esac
    case "$(uname -m)" in
        x86_64|amd64)  arch="x86_64" ;;
        aarch64|arm64) arch="aarch64" ;;
        *) echo "install: arquitectura não suportada: $(uname -m)" >&2; return 1 ;;
    esac
    printf '%s-%s' "$arch" "$os"
}

install_binary() {
    local bin="$1"
    if [ ! -x "$bin" ]; then
        echo "install: binário não encontrado: $bin" >&2
        return 1
    fi
    mkdir -p "$install_dir"
    cp "$bin" "$install_dir/ktu"
    chmod +x "$install_dir/ktu"
    echo "install: ktu instalado em $install_dir/ktu" >&2
    case ":$PATH:" in
        *":$install_dir:"*) ;;
        *) echo "install: acrescente $install_dir ao PATH" >&2 ;;
    esac
}

install_from_source() {
    local src="${KTU_SOURCE_DIR:-$PWD}"
    if [ ! -f "$src/Cargo.toml" ] || [ ! -d "$src/crates/ktu-cli" ]; then
        echo "install: KTU_FROM_SOURCE=1 precisa de um checkout do ktu em $src" >&2
        return 1
    fi
    echo "install: a compilar do código-fonte em $src" >&2
    ( cd "$src" && cargo build --release -p ktu-cli --locked )
    install_binary "$src/target/release/ktu"
}

install_from_release() {
    local triple tag ver archive url tmp json api
    triple="$(detect_triple)"
    tag=""
    if [ "$version" = "latest" ]; then
        api="https://api.github.com/repos/${repo}/releases/latest"
        if json="$(curl -fsSL "$api" 2>/dev/null)"; then
            tag="$(printf '%s' "$json" | sed -n 's/.*"tag_name": *"\([^"]*\)".*/\1/p' | head -1)"
        fi
    else
        tag="$version"
    fi
    if [ -z "$tag" ]; then
        echo "install: não foi possível resolver a versão (use KTU_FROM_SOURCE=1)" >&2
        return 1
    fi
    ver="${tag#v}"
    archive="ktu-${ver}-${triple}.tar.gz"
    url="${KTU_RELEASE_BASE:-https://github.com/${repo}/releases/download/${tag}}/${archive}"
    tmp="$(mktemp -d)"
    trap 'rm -rf "${tmp:-}"' EXIT
    echo "install: a descarregar ${url}" >&2
    if ! curl -fsSL "$url" -o "$tmp/$archive"; then
        echo "install: release indisponível; tente KTU_FROM_SOURCE=1" >&2
        return 1
    fi
    if curl -fsSL "${url}.sha256" -o "$tmp/${archive}.sha256" 2>/dev/null; then
        if command -v sha256sum >/dev/null 2>&1; then
            ( cd "$tmp" && sha256sum -c "${archive}.sha256" )
        else
            ( cd "$tmp" && shasum -a 256 -c "${archive}.sha256" )
        fi
    fi
    tar -C "$tmp" -xzf "$tmp/$archive"
    install_binary "$tmp/ktu-${ver}-${triple}/ktu"
}

if [ "${KTU_FROM_SOURCE:-0}" = "1" ]; then
    install_from_source
else
    install_from_release
fi
