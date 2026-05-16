#!/usr/bin/env bash
set -euo pipefail

TOOLS_DIR="$HOME/.ecaz/hardening-tools"
RUSTUP_BIN="${RUSTUP_BIN:-/opt/homebrew/opt/rustup/bin/rustup}"
RUSTUP_CARGO="${RUSTUP_CARGO:-/opt/homebrew/opt/rustup/bin/cargo}"
INSTALL_RUDRA=false
INSTALL_MIRAI=false
INSTALL_FLUX=false
CHECK_ONLY=false
LOG_FILE=""

usage() {
  cat <<'EOF'
usage: scripts/install_hardening_tools.sh [--all] [--rudra] [--mirai] [--flux] [--check] [--tools-dir DIR] [--log-file FILE]

Installs or checks optional hardening tools from their upstream source layouts.
Source checkouts live under ~/.ecaz/hardening-tools by default so the tools can
be reused across future tasks without becoming repository content.
EOF
}

need_cmd() {
  local cmd="$1"
  if ! command -v "$cmd" >/dev/null 2>&1; then
    echo "missing required installer command: $cmd" >&2
    exit 127
  fi
}

clone_or_update() {
  local url="$1"
  local dir="$2"
  if [ -d "$dir/.git" ]; then
    git -C "$dir" fetch --depth 1 origin
    git -C "$dir" checkout FETCH_HEAD
  else
    git clone --depth 1 "$url" "$dir"
  fi
}

check_tool() {
  local label="$1"
  local cmd="$2"
  if command -v "$cmd" >/dev/null 2>&1; then
    printf '%-12s %s\n' "$label:" "$(command -v "$cmd")"
  else
    printf '%-12s missing\n' "$label:"
  fi
}

install_rudra() {
  need_cmd docker
  need_cmd git
  need_cmd python3
  mkdir -p "$TOOLS_DIR"
  if ! docker image inspect rudra:latest >/dev/null 2>&1; then
    docker pull ghcr.io/sslab-gatech/rudra:master
    docker tag ghcr.io/sslab-gatech/rudra:master rudra:latest
  fi
  clone_or_update https://github.com/sslab-gatech/Rudra "$TOOLS_DIR/Rudra"
  python3 "$TOOLS_DIR/Rudra/setup_rudra_runner_home.py" "$TOOLS_DIR/rudra-home"
  echo "rudra docker helper ready: $TOOLS_DIR/Rudra/docker-helper/docker-cargo-rudra"
}

install_mirai() {
  need_cmd git
  if [ ! -x "$RUSTUP_BIN" ] || [ ! -x "$RUSTUP_CARGO" ]; then
    echo "missing rustup cargo shim; install rustup with brew install rustup" >&2
    exit 127
  fi
  need_cmd cmake
  mkdir -p "$TOOLS_DIR"
  clone_or_update https://github.com/endorlabs/MIRAI "$TOOLS_DIR/MIRAI"
  "$RUSTUP_BIN" toolchain install nightly-2025-01-10 \
    --component clippy \
    --component rustfmt \
    --component rustc-dev \
    --component rust-src \
    --component rust-std \
    --component llvm-tools-preview
  (
    cd "$TOOLS_DIR/MIRAI"
    bash install_mirai.sh
  )
  cargo mirai --help >/dev/null
}

install_flux() {
  need_cmd git
  need_cmd curl
  need_cmd tar
  if [ ! -x "$RUSTUP_BIN" ] || [ ! -x "$RUSTUP_CARGO" ]; then
    echo "missing rustup cargo shim; install rustup with brew install rustup" >&2
    exit 127
  fi
  need_cmd z3
  mkdir -p "$TOOLS_DIR"
  if ! command -v fixpoint >/dev/null 2>&1; then
    host="$("$RUSTUP_CARGO" -vV | awk '/host:/ {print $2}')"
    case "$host" in
      aarch64-apple-darwin)
        fixpoint_asset="fixpoint-aarch64-apple-darwin.tar.gz"
        ;;
      x86_64-unknown-linux-gnu)
        fixpoint_asset="fixpoint-x86_64-linux-gnu.tar.gz"
        ;;
      *)
        echo "missing fixpoint and no known liquid-fixpoint binary asset for host: $host" >&2
        exit 127
        ;;
    esac
    fixpoint_dir="$TOOLS_DIR/liquid-fixpoint"
    mkdir -p "$fixpoint_dir"
    curl -L \
      -o "$fixpoint_dir/$fixpoint_asset" \
      "https://github.com/ucsd-progsys/liquid-fixpoint/releases/download/nightly/$fixpoint_asset"
    tar -xzf "$fixpoint_dir/$fixpoint_asset" -C "$fixpoint_dir"
    fixpoint_bin="$(find "$fixpoint_dir" -type f -name fixpoint | head -1)"
    if [ -z "$fixpoint_bin" ]; then
      echo "downloaded liquid-fixpoint asset but could not find fixpoint binary" >&2
      exit 127
    fi
    mkdir -p "$HOME/.cargo/bin"
    cp "$fixpoint_bin" "$HOME/.cargo/bin/fixpoint"
    chmod +x "$HOME/.cargo/bin/fixpoint"
  fi
  clone_or_update https://github.com/flux-rs/flux "$TOOLS_DIR/flux"
  (
    cd "$TOOLS_DIR/flux"
    cargo xtask install
  )
  cargo flux --version
}

while [ "$#" -gt 0 ]; do
  case "$1" in
    --all)
      INSTALL_RUDRA=true
      INSTALL_MIRAI=true
      INSTALL_FLUX=true
      shift
      ;;
    --rudra)
      INSTALL_RUDRA=true
      shift
      ;;
    --mirai)
      INSTALL_MIRAI=true
      shift
      ;;
    --flux)
      INSTALL_FLUX=true
      shift
      ;;
    --check)
      CHECK_ONLY=true
      shift
      ;;
    --tools-dir)
      TOOLS_DIR="${2:-}"
      if [ -z "$TOOLS_DIR" ]; then
        echo "missing value for --tools-dir" >&2
        exit 2
      fi
      shift 2
      ;;
    --log-file)
      LOG_FILE="${2:-}"
      if [ -z "$LOG_FILE" ]; then
        echo "missing value for --log-file" >&2
        exit 2
      fi
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "unknown flag: $1" >&2
      usage >&2
      exit 2
      ;;
  esac
done

export PATH="$HOME/.cargo/bin:$(dirname "$RUSTUP_CARGO"):/opt/homebrew/bin:$PATH"

if [ -n "$LOG_FILE" ]; then
  mkdir -p "$(dirname "$LOG_FILE")"
  exec > >(tee "$LOG_FILE") 2>&1
fi

if [ "$CHECK_ONLY" = true ]; then
  check_tool cargo-audit cargo-audit
  check_tool cargo-deny cargo-deny
  check_tool cargo-vet cargo-vet
  check_tool cargo-geiger cargo-geiger
  check_tool cargo-careful cargo-careful
  check_tool cargo-fuzz cargo-fuzz
  check_tool cargo-afl cargo-afl
  check_tool cargo-kani cargo-kani
  check_tool sqlsmith sqlsmith
  check_tool cargo-mirai cargo-mirai
  check_tool cargo-flux cargo-flux
  if [ -x "$TOOLS_DIR/Rudra/docker-helper/docker-cargo-rudra" ]; then
    printf '%-12s %s\n' "rudra:" "$TOOLS_DIR/Rudra/docker-helper/docker-cargo-rudra"
  else
    printf '%-12s missing\n' "rudra:"
  fi
  exit 0
fi

if [ "$INSTALL_RUDRA" = false ] && [ "$INSTALL_MIRAI" = false ] && [ "$INSTALL_FLUX" = false ]; then
  usage >&2
  exit 2
fi

if [ "$INSTALL_RUDRA" = true ]; then
  install_rudra
fi
if [ "$INSTALL_MIRAI" = true ]; then
  install_mirai
fi
if [ "$INSTALL_FLUX" = true ]; then
  install_flux
fi
