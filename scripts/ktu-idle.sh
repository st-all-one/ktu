#!/usr/bin/env bash
# ktu-idle — worker de drenagem de embeddings e gestor do agendador (H8).
#
# O ktu não tem daemon: `ktu memory drain --digest` é explícito. Este worker mantém um
# servidor de embeddings local de pé e drena a fila dos projectos cadastrados, para que o
# recall semântico não dependa de o humano correr o dreno à mão.
#
# Agendador: `systemd --user` (Linux) ou `launchd` (macOS); sem nenhum dos dois, imprime a
# linha de cron equivalente. O servidor corre como unidade **persistente** (`ktu-embed`);
# o worker só drena (e sobe um servidor efémero se o persistente estiver fora).
#
# Uso:
#   ktu-idle install [--project DIR] [--port N] [--model PATH] [--every DUR] [--no-deps] [--dry-run]
#   ktu-idle subscribe [--project DIR]      # cadastra um projecto (multi-projecto)
#   ktu-idle unsubscribe [--project DIR]    # descadastra (mantém o sistema instalado)
#   ktu-idle status                         # saúde: agendador, servidor, fila por projecto
#   ktu-idle reconcile [--project DIR]      # alinha memory.toml ao worker instalado
#   ktu-idle uninstall [--keep-model|--remove-model]
#   ktu-idle run [PROJECTO...]              # corpo do worker (o agendador chama isto)
#
# O llama.cpp **não** é descarregado: é pré-requisito do provider (o ktu fala com o
# `llama-server`); o worker localiza-o no PATH ou em $KTU_LLAMA e, se faltar, imprime o guia.
# O GGUF de embeddings (o mesmo modelo do knudge) é descarregado com **revisão pinada** e
# SHA-256 verificado; `--no-deps` salta a descarga (falha se faltar).
#
# Segurança: idempotente; nunca usa `rm` (move para ${XDG_CACHE_HOME:-~/.cache}/ktu/trash).
# Sem `set -e` de propósito: passos opcionais degradam com aviso (o agendador é best-effort).
set -uo pipefail

PROG="ktu-idle"
SELF="${BASH_SOURCE[0]:-}"
XDG_CONFIG="${XDG_CONFIG_HOME:-$HOME/.config}"
XDG_CACHE="${XDG_CACHE_HOME:-$HOME/.cache}"
CONF_DIR="$XDG_CONFIG/ktu"
CONF="$CONF_DIR/idle.conf"
PROJECTS="$CONF_DIR/idle.projects"
UNIT_DIR="$XDG_CONFIG/systemd/user"
EMBED_UNIT="ktu-embed"
IDLE_UNIT="ktu-idle"
LA_DIR="$HOME/Library/LaunchAgents"
LA_EMBED_LABEL="local.ktu.embed"
LA_IDLE_LABEL="local.ktu.idle"
LA_EMBED_PLIST="$LA_DIR/$LA_EMBED_LABEL.plist"
LA_IDLE_PLIST="$LA_DIR/$LA_IDLE_LABEL.plist"
BIN_DIR="${KTU_BIN_DIR:-$HOME/.local/bin}"
TRASH="$XDG_CACHE/ktu/trash"
KTU="${KTU_BIN:-$BIN_DIR/ktu}"
LLAMA="${KTU_LLAMA:-$BIN_DIR/llama}"
LOG_DIR="$XDG_CACHE/ktu"
LOG="$LOG_DIR/idle.log"
DEFAULT_MODEL="$CONF_DIR/embeddinggemma-2-Q8_0.gguf"
MODEL_ID="google/embeddinggemma-2"
MODEL_DIMENSIONS=768
DEFAULT_PORT=8889
DEFAULT_EVERY=1h
# Revisão **pinada** (nunca `/resolve/main/`) e SHA-256 do GGUF.
MODEL_REVISION="bfcd298762cc34d0357ece5ebdd31791a3a374d8"
MODEL_FILE="embeddinggemma-2-Q8_0.gguf"
MODEL_SHA256="2188ac1deca4b77dffefd603c2776a9d76d9d74ec01841392982ebb840b09135"
MODEL_URL="https://huggingface.co/ggml-org/embeddinggemma-2-GGUF/resolve/$MODEL_REVISION/$MODEL_FILE"
SCHEDULER=""
STARTED_PID=""

log() { printf '%s: %s\n' "$PROG" "$*" >&2; }
warn() { printf '%s: aviso: %s\n' "$PROG" "$*" >&2; }
next_cmd() { printf '  próximos: %s\n' "$*" >&2; }
die() {
    log "$*"
    exit 1
}

# Guia manual (quando falta uma dependência que não descarregamos).
die_with_guide() {
    log "$*"
    cat >&2 <<'GUIDE'
  Instale o llama.cpp (traz o `llama-server` de embeddings):
    curl -fsSL https://llama.app/install.sh | sh      # ou: brew install llama.cpp
  Depois aponte o binário:  export KTU_LLAMA=/caminho/para/llama
  E repita:                 ktu memory drain service --install
GUIDE
    exit 1
}

# Move para o lixo recuperável — **nunca** apaga.
trash() {
    [ -e "$1" ] || return 0
    mkdir -p "$TRASH"
    mv "$1" "$TRASH/$(basename "$1").$(date +%Y%m%d%H%M%S).$$" 2>/dev/null || true
}

# SHA-256 em hex (supply-chain): usa o primeiro verificador disponível.
sha256_of() {
    local file=$1
    if command -v sha256sum >/dev/null 2>&1; then
        sha256sum "$file" | awk '{print $1}'
    elif command -v shasum >/dev/null 2>&1; then
        shasum -a 256 "$file" | awk '{print $1}'
    elif command -v openssl >/dev/null 2>&1; then
        openssl dgst -sha256 "$file" | awk '{print $NF}'
    else
        return 1
    fi
}

# Descarrega uma URL para um destino, com curl e queda para wget.
fetch() {
    local url=$1 dest=$2
    if command -v curl >/dev/null 2>&1; then
        curl -fL --proto '=https' --tlsv1.2 -o "$dest" "$url" && return 0
    fi
    if command -v wget >/dev/null 2>&1; then
        wget -q -O "$dest" "$url" && return 0
    fi
    return 1
}

# Converte `30m`/`1h`/`1d` em segundos (para o `StartInterval` do launchd).
duration_seconds() {
    local raw=$1 digits unit
    digits="${raw//[!0-9]/}"
    [ -n "$digits" ] || die "duração inválida: $raw (use 30m, 1h, 1d)"
    unit="${raw//[0-9]/}"
    case "$unit" in
        "" | s) printf '%s\n' "$digits" ;;
        m) printf '%s\n' "$((digits * 60))" ;;
        h) printf '%s\n' "$((digits * 3600))" ;;
        d) printf '%s\n' "$((digits * 86400))" ;;
        *) die "duração inválida: $raw (use 30m, 1h, 1d)" ;;
    esac
}

# ---------- configuração ----------
conf_value() {
    [ -f "$CONF" ] || return 1
    grep -m1 "^$1=" "$CONF" 2>/dev/null | cut -d= -f2-
}

write_conf() {
    mkdir -p "$CONF_DIR"
    cat >"$CONF" <<EOF
# gerado por $PROG ($(date +%Y-%m-%d)) — porta, modelo e período do worker
port=$PORT
model=$MODEL
every=$EVERY
EOF
}

# Projectos cadastrados, em ordem (um caminho absoluto por linha).
projects_list() {
    [ -f "$PROJECTS" ] || return 0
    grep -v '^[[:space:]]*$' "$PROJECTS" 2>/dev/null
}

subscribe_project() {
    local proj=$1
    [ -n "$proj" ] || return 0
    mkdir -p "$CONF_DIR"
    if projects_list | grep -Fxq "$proj"; then
        log "projecto já cadastrado: $proj"
        return 0
    fi
    printf '%s\n' "$proj" >>"$PROJECTS"
    log "projecto cadastrado: $proj"
}

unsubscribe_project() {
    local proj=$1 tmp
    [ -n "$proj" ] || return 0
    [ -f "$PROJECTS" ] || return 0
    tmp="$PROJECTS.$$"
    projects_list | grep -Fxv "$proj" >"$tmp" || true
    mv "$tmp" "$PROJECTS"
    log "projecto descadastrado: $proj"
}

# ---------- memory.toml (uma só verdade: mesmas chaves do knudge) ----------
align_memory_toml() {
    local proj=$1 path want
    path="$proj/.agents/memory.toml"
    want="[embeddings]
provider = \"http\"
endpoint = \"http://127.0.0.1:$PORT/v1/embeddings\"
model = \"$MODEL_ID\"
revision = \"main\"
dimensions = $MODEL_DIMENSIONS
prefix = \"auto\"
batch = 32
"
    mkdir -p "$(dirname "$path")"
    if [ -f "$path" ] && [ "$(cat "$path" 2>/dev/null)" = "$want" ]; then
        log "memory.toml já alinhado: $proj"
        return 0
    fi
    printf '%s' "$want" >"$path"
    log "memory.toml alinhado ao worker: $proj"
    next_cmd "ktu memory drain --force   # reindexar com o modelo novo"
}

# ---------- agendador ----------
detect_scheduler() {
    if command -v systemctl >/dev/null 2>&1 && systemctl --user show-environment >/dev/null 2>&1; then
        SCHEDULER="systemd"
    elif [ "$(uname -s)" = "Darwin" ] && command -v launchctl >/dev/null 2>&1; then
        SCHEDULER="launchd"
    else
        SCHEDULER="none"
    fi
}

write_systemd_units() {
    mkdir -p "$UNIT_DIR" "$LOG_DIR"
    cat >"$UNIT_DIR/$IDLE_UNIT.service" <<EOF
[Unit]
Description=ktu: drena a fila de embeddings dos projectos cadastrados
Documentation=file:$CONF

[Service]
Type=oneshot
ExecStart=$BIN_DIR/$IDLE_UNIT run
Nice=10
EOF
    cat >"$UNIT_DIR/$IDLE_UNIT.timer" <<EOF
[Unit]
Description=ktu: dreno periódico da fila de embeddings

[Timer]
OnActiveSec=5min
OnUnitActiveSec=$EVERY
AccuracySec=1min
Persistent=true
Unit=$IDLE_UNIT.service

[Install]
WantedBy=timers.target
EOF
    cat >"$UNIT_DIR/$EMBED_UNIT.service" <<EOF
[Unit]
Description=ktu: servidor de embeddings local (llama.cpp)
Documentation=file:$CONF
After=network.target

[Service]
Type=simple
ExecStart=$LLAMA serve -m $MODEL --embeddings --pooling mean --host 127.0.0.1 --port $PORT -b 2048 -ub 2048
Restart=on-failure
RestartSec=5
StandardOutput=append:$LOG_DIR/embed.log
StandardError=append:$LOG_DIR/embed.log

[Install]
WantedBy=default.target
EOF
    systemctl --user daemon-reload >/dev/null 2>&1 || true
    systemctl --user enable --now "$EMBED_UNIT.service" >/dev/null 2>&1 || true
    systemctl --user enable --now "$IDLE_UNIT.timer" >/dev/null 2>&1 || true
}

write_launchd_plists() {
    mkdir -p "$LA_DIR" "$LOG_DIR"
    local interval
    interval="$(duration_seconds "$EVERY")"
    cat >"$LA_IDLE_PLIST" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>$LA_IDLE_LABEL</string>
    <key>ProgramArguments</key>
    <array>
        <string>$BIN_DIR/$IDLE_UNIT</string>
        <string>run</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>StartInterval</key>
    <integer>$interval</integer>
    <key>StandardOutPath</key>
    <string>$LOG_DIR/idle.log</string>
    <key>StandardErrorPath</key>
    <string>$LOG_DIR/idle.err.log</string>
</dict>
</plist>
EOF
    cat >"$LA_EMBED_PLIST" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>$LA_EMBED_LABEL</string>
    <key>ProgramArguments</key>
    <array>
        <string>$LLAMA</string>
        <string>serve</string>
        <string>-m</string>
        <string>$MODEL</string>
        <string>--embeddings</string>
        <string>--pooling</string>
        <string>mean</string>
        <string>--host</string>
        <string>127.0.0.1</string>
        <string>--port</string>
        <string>$PORT</string>
        <string>-b</string>
        <string>2048</string>
        <string>-ub</string>
        <string>2048</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
    <key>StandardOutPath</key>
    <string>$LOG_DIR/embed.log</string>
    <key>StandardErrorPath</key>
    <string>$LOG_DIR/embed.log</string>
</dict>
</plist>
EOF
    launchd_load "$LA_EMBED_PLIST" "$LA_EMBED_LABEL"
    launchd_load "$LA_IDLE_PLIST" "$LA_IDLE_LABEL"
}

launchd_load() {
    launchctl unload "$1" >/dev/null 2>&1 || true
    launchctl load "$1" >/dev/null 2>&1 || warn "não foi possível carregar $1"
}

launchd_unload() {
    launchctl unload "$1" >/dev/null 2>&1 || true
}

# Estado legível de uma unidade systemd (`is-active` sai não-zero quando inativa).
unit_state() {
    local state
    state="$(systemctl --user is-active "$1" 2>/dev/null)"
    case "$state" in
        active) printf ativo ;;
        inactive) printf inativo ;;
        "") printf ausente ;;
        *) printf '%s' "$state" ;;
    esac
}

scheduler_status() {
    case "$SCHEDULER" in
        systemd)
            printf 'timer: %s\n' "$(unit_state "$IDLE_UNIT.timer")"
            printf 'servidor (unit): %s\n' "$(unit_state "$EMBED_UNIT.service")"
            ;;
        launchd)
            printf 'agente idle: %s\n' "$(launchctl print "gui/$(id -u)/$LA_IDLE_LABEL" >/dev/null 2>&1 && printf carregado || printf ausente)"
            printf 'agente servidor: %s\n' "$(launchctl print "gui/$(id -u)/$LA_EMBED_LABEL" >/dev/null 2>&1 && printf carregado || printf ausente)"
            ;;
        *) printf 'agendador: nenhum (use cron: %s run)\n' "$BIN_DIR/$IDLE_UNIT" ;;
    esac
}

scheduler_remove() {
    case "$SCHEDULER" in
        systemd)
            systemctl --user disable --now "$IDLE_UNIT.timer" >/dev/null 2>&1 || true
            systemctl --user disable --now "$EMBED_UNIT.service" >/dev/null 2>&1 || true
            trash "$UNIT_DIR/$IDLE_UNIT.service"
            trash "$UNIT_DIR/$IDLE_UNIT.timer"
            trash "$UNIT_DIR/$EMBED_UNIT.service"
            systemctl --user daemon-reload >/dev/null 2>&1 || true
            ;;
        launchd)
            launchd_unload "$LA_IDLE_PLIST" "$LA_IDLE_LABEL"
            launchd_unload "$LA_EMBED_PLIST" "$LA_EMBED_LABEL"
            trash "$LA_IDLE_PLIST"
            trash "$LA_EMBED_PLIST"
            ;;
    esac
}

# ---------- servidor e modelo ----------
resolve_llama() {
    [ -x "$LLAMA" ] && return 0
    local found
    found="$(command -v llama 2>/dev/null || true)"
    [ -z "$found" ] && [ -x "$HOME/.local/bin/llama" ] && found="$HOME/.local/bin/llama"
    [ -n "$found" ] && LLAMA="$found"
    return 0
}

ensure_model() {
    [ -f "$MODEL" ] && return 0
    if [ "$NO_DEPS" = "1" ]; then
        die "modelo ausente em $MODEL e --no-deps foi pedido"
    fi
    local tmp="$XDG_CACHE/ktu/model.$$"
    mkdir -p "$(dirname "$tmp")" "$(dirname "$MODEL")"
    log "baixando o modelo de embeddings ($MODEL_ID, revisão pinada)..."
    if ! fetch "$MODEL_URL" "$tmp"; then
        trash "$tmp"
        die "não foi possível baixar $MODEL_URL"
    fi
    local got
    got="$(sha256_of "$tmp" || true)"
    if [ "$got" != "$MODEL_SHA256" ]; then
        trash "$tmp"
        die "SHA-256 do modelo não casa (esperado $MODEL_SHA256, obtido ${got:-<indisponível>}); nada foi instalado"
    fi
    mv "$tmp" "$MODEL"
    log "modelo instalado: $MODEL"
}

ensure_server() {
    local health="http://127.0.0.1:$PORT/health"
    local probe=1
    mkdir -p "$LOG_DIR"
    command -v curl >/dev/null 2>&1 || probe=0
    if [ "$probe" = "1" ] && curl -fsS "$health" >/dev/null 2>&1; then
        return 0
    fi
    resolve_llama
    [ -x "$LLAMA" ] || die_with_guide "llama não encontrado (defina KTU_LLAMA)"
    log "servidor de embeddings fora; subindo um efémero na porta $PORT"
    "$LLAMA" serve -m "$MODEL" --embeddings --pooling mean \
        --host 127.0.0.1 --port "$PORT" -b 2048 -ub 2048 >>"$LOG" 2>&1 &
    STARTED_PID=$!
    if [ "$probe" = "0" ]; then
        warn "curl ausente: sem sonda de saúde (o dreno falha alto se o servidor não subir)"
        return 0
    fi
    local attempt=1
    while [ "$attempt" -le 60 ]; do
        curl -fsS "$health" >/dev/null 2>&1 && return 0
        sleep 1
        attempt=$((attempt + 1))
    done
    die "servidor de embeddings não subiu (veja $LOG)"
}

cleanup() {
    [ -n "$STARTED_PID" ] && kill "$STARTED_PID" 2>/dev/null
    return 0
}

# ---------- acções ----------
read_settings() {
    PORT="$(conf_value port || true)"
    MODEL="$(conf_value model || true)"
    EVERY="$(conf_value every || true)"
    [ -n "$PORT" ] || PORT="$DEFAULT_PORT"
    [ -n "$MODEL" ] || MODEL="$DEFAULT_MODEL"
    [ -n "$EVERY" ] || EVERY="$DEFAULT_EVERY"
}

install_worker() {
    read_settings
    [ -n "$PORT_ARG" ] && PORT="$PORT_ARG"
    [ -n "$MODEL_ARG" ] && MODEL="$MODEL_ARG"
    [ -n "$EVERY_ARG" ] && EVERY="$EVERY_ARG"
    detect_scheduler
    if [ "$DRY_RUN" = "1" ]; then
        cat <<EOF
dry-run: $PROG install
  binário do worker: $BIN_DIR/$IDLE_UNIT (a partir de $SELF)
  config: $CONF (port=$PORT every=$EVERY)
  modelo: $MODEL ($MODEL_ID, revisão $MODEL_REVISION)
  agendador: $SCHEDULER
  projecto: $PROJECT
EOF
        return 0
    fi
    resolve_llama
    [ -x "$LLAMA" ] || die_with_guide "llama não encontrado (defina KTU_LLAMA)"
    ensure_model
    mkdir -p "$BIN_DIR" "$CONF_DIR" "$LOG_DIR"
    if [ -n "$SELF" ] && [ -f "$SELF" ]; then
        install -m 0755 "$SELF" "$BIN_DIR/$IDLE_UNIT"
    fi
    write_conf
    case "$SCHEDULER" in
        systemd) write_systemd_units ;;
        launchd) write_launchd_plists ;;
        none) warn "sem systemd --user/launchd; agende à mão: $BIN_DIR/$IDLE_UNIT run" ;;
    esac
    subscribe_project "$PROJECT"
    align_memory_toml "$PROJECT"
    log "worker instalado (porta $PORT, período $EVERY)"
    next_cmd "verifique com 'ktu memory drain service --status'"
}

do_subscribe() {
    read_settings
    subscribe_project "$PROJECT"
    align_memory_toml "$PROJECT"
}

do_unsubscribe() {
    unsubscribe_project "$PROJECT"
}

do_reconcile() {
    read_settings
    align_memory_toml "$PROJECT"
}

do_status() {
    read_settings
    detect_scheduler
    printf 'worker: %s\n' "$PROG"
    printf 'config: %s\n' "$CONF"
    printf 'modelo: %s\n' "$MODEL"
    printf 'porta: %s · período: %s\n' "$PORT" "$EVERY"
    scheduler_status
    if command -v curl >/dev/null 2>&1; then
        printf 'servidor: %s\n' "$(curl -fsS "http://127.0.0.1:$PORT/health" >/dev/null 2>&1 && printf ativo || printf inativo)"
    fi
    local proj out
    while IFS= read -r proj; do
        [ -n "$proj" ] || continue
        out="$(cd "$proj" 2>/dev/null && "$KTU" memory drain --status 2>&1)" || out="(inacessível)"
        printf 'projecto %s\n%s\n' "$proj" "$out"
    done < <(projects_list)
}

do_uninstall() {
    read_settings
    detect_scheduler
    scheduler_remove
    trash "$BIN_DIR/$IDLE_UNIT"
    if [ "$REMOVE_MODEL" = "1" ]; then
        trash "$MODEL"
        log "modelo movido para o lixo: $TRASH"
    else
        log "modelo preservado: $MODEL"
    fi
    log "worker removido (config preservada: $CONF)"
}

drain_all() {
    local rc=0 proj out
    for proj in "$@"; do
        out="$(cd "$proj" 2>/dev/null && "$KTU" memory drain --digest 2>&1)" || {
            log "falha no dreno de $proj: $out"
            rc=1
            continue
        }
        printf '%s: %s\n' "$proj" "$out"
    done
    return $rc
}

do_run() {
    read_settings
    resolve_llama
    [ -x "$LLAMA" ] || die_with_guide "llama não encontrado (defina KTU_LLAMA)"
    [ -f "$MODEL" ] || die "modelo ausente em $MODEL (rode 'ktu memory drain service --install')"
    ensure_server
    trap cleanup EXIT
    if [ "$#" -gt 0 ]; then
        drain_all "$@"
    else
        local projects=()
        while IFS= read -r proj; do
            [ -n "$proj" ] && projects+=("$proj")
        done < <(projects_list)
        if [ "${#projects[@]}" -eq 0 ]; then
            log "nenhum projecto cadastrado; nada a drenar"
            return 0
        fi
        drain_all "${projects[@]}"
    fi
}

# ---------- argumentos ----------
ACTION=""
PROJECT=""
PORT_ARG=""
MODEL_ARG=""
EVERY_ARG=""
NO_DEPS="0"
REMOVE_MODEL="0"
DRY_RUN="0"
RUN_PROJECTS=()

parse_args() {
    while [ "$#" -gt 0 ]; do
        case "$1" in
            --project) PROJECT="${2:-}"; shift 2 ;;
            --port) PORT_ARG="${2:-}"; shift 2 ;;
            --model) MODEL_ARG="${2:-}"; shift 2 ;;
            --every) EVERY_ARG="${2:-}"; shift 2 ;;
            --no-deps) NO_DEPS="1"; shift ;;
            --remove-model) REMOVE_MODEL="1"; shift ;;
            --keep-model) REMOVE_MODEL="0"; shift ;;
            --dry-run) DRY_RUN="1"; shift ;;
            -h | --help) ACTION="help"; shift ;;
            -*) die "opção desconhecida: $1" ;;
            *) RUN_PROJECTS+=("$1"); shift ;;
        esac
    done
}

main() {
    parse_args "$@"
    if [ -z "$PROJECT" ]; then
        PROJECT="$(pwd)"
    fi
    case "$ACTION" in
        install) install_worker ;;
        subscribe) do_subscribe ;;
        unsubscribe) do_unsubscribe ;;
        reconcile) do_reconcile ;;
        status) do_status ;;
        uninstall) do_uninstall ;;
        run) do_run "${RUN_PROJECTS[@]}" ;;
        help | "") sed -n '2,30p' "$SELF" >&2 2>/dev/null || true ;;
        *) die "acção desconhecida: $ACTION" ;;
    esac
}

# A primeira palavra não-flag é a acção (o CLI passa-a como `$1`).
if [ "$#" -gt 0 ]; then
    case "$1" in
        install | subscribe | unsubscribe | reconcile | status | uninstall | run)
            ACTION="$1"
            shift
            ;;
    esac
fi
main "$@"
