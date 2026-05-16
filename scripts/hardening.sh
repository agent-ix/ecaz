#!/usr/bin/env bash
set -euo pipefail

export PATH="${HOME}/.cargo/bin:${PATH}"
RUSTUP_CARGO="${RUSTUP_CARGO:-/opt/homebrew/opt/rustup/bin/cargo}"
RUSTUP_BIN="${RUSTUP_BIN:-/opt/homebrew/opt/rustup/bin/rustup}"
ECAZ_HARDENING_TOOLS_DIR="${ECAZ_HARDENING_TOOLS_DIR:-$HOME/.ecaz/hardening-tools}"

usage() {
  cat <<'EOF'
usage: scripts/hardening.sh <lane>

Local-first hardening lanes. Each lane checks for optional tooling before it
runs and prints install/setup guidance when the tool is missing.

lane flags:
  rudra --manifest-path CARGO_TOML
  fuzz-all-short --seconds N
  sqlsmith-pg18 --dsn LIBPQ_DSN
  any lane --log-file FILE
EOF
}

need_cmd() {
  local cmd="$1"
  local install="$2"
  if ! command -v "$cmd" >/dev/null 2>&1; then
    cat >&2 <<EOF
missing optional hardening tool: $cmd
install/setup:
  $install
EOF
    exit 127
  fi
}

need_nightly() {
  if [ ! -x "$RUSTUP_CARGO" ]; then
    cat >&2 <<EOF
missing rustup cargo shim: $RUSTUP_CARGO
install/setup:
  brew install rustup
EOF
    exit 127
  fi
  if ! "$RUSTUP_CARGO" +nightly --version >/dev/null 2>&1; then
    cat >&2 <<'EOF'
missing Rust nightly toolchain
install/setup:
  Install rustup from https://rustup.rs, then run:
  rustup toolchain install nightly
EOF
    exit 127
  fi
}

need_nightly_miri() {
  need_nightly
  if [ ! -x "$RUSTUP_BIN" ]; then
    cat >&2 <<EOF
missing rustup binary: $RUSTUP_BIN
install/setup:
  brew install rustup
EOF
    exit 127
  fi
  if ! "$RUSTUP_BIN" which --toolchain nightly cargo-miri >/dev/null 2>&1; then
    cat >&2 <<'EOF'
missing nightly miri component
install/setup:
  rustup +nightly component add miri
EOF
    exit 127
  fi
}

nightly_path() {
  PATH="$(dirname "$RUSTUP_BIN"):$PATH"
  export PATH
  RUSTUP_TOOLCHAIN=nightly
  export RUSTUP_TOOLCHAIN
}

host_triple() {
  rustc -vV | awk '/host:/ {print $2}'
}

run_sanitized_lib_tests() {
  local sanitizer="$1"
  need_nightly
  nightly_path
  case "$(host_triple):$sanitizer" in
    aarch64-apple-darwin:leak|aarch64-apple-darwin:memory)
      echo "skipping ${sanitizer} sanitizer: rustc does not support it for $(host_triple)"
      return 0
      ;;
  esac
  RUSTFLAGS="-Zsanitizer=${sanitizer}"
  export RUSTFLAGS
  if [ "$sanitizer" = "thread" ]; then
    cargo -Z build-std test --manifest-path hardening/careful/Cargo.toml --target "$(host_triple)" --lib
  else
    cargo test --manifest-path hardening/careful/Cargo.toml --target "$(host_triple)" --lib
  fi
}

run_sanitized_pg18_tests() {
  local sanitizer="$1"
  need_nightly
  nightly_path
  RUSTFLAGS="-Zsanitizer=${sanitizer}"
  export RUSTFLAGS
  cargo pgrx test pg18
}

mac_dynamic_lookup_config=()
if [ "$(host_triple)" = "aarch64-apple-darwin" ]; then
  mac_dynamic_lookup_config=(
    --config 'target.aarch64-apple-darwin.rustflags=["-C","link-arg=-undefined","-C","link-arg=dynamic_lookup"]'
  )
fi

careful_cargo_config=(
  --config 'profile.dev.lto=false'
  --config 'profile.test.lto=false'
)
careful_cargo_config+=("${mac_dynamic_lookup_config[@]}")

lane="${1:-}"
if [ -z "$lane" ]; then
  usage >&2
  exit 2
fi
shift

log_file=""
remaining_args=()
while [ "$#" -gt 0 ]; do
  case "$1" in
    --log-file)
      log_file="${2:-}"
      if [ -z "$log_file" ]; then
        echo "missing value for --log-file" >&2
        exit 2
      fi
      shift 2
      ;;
    *)
      remaining_args+=("$1")
      shift
      ;;
  esac
done
if [ "${#remaining_args[@]}" -gt 0 ]; then
  set -- "${remaining_args[@]}"
else
  set --
fi

if [ -n "$log_file" ]; then
  mkdir -p "$(dirname "$log_file")"
  if [ "${ECAZ_HARDENING_LOG_ACTIVE:-}" != "1" ]; then
    ECAZ_HARDENING_LOG_ACTIVE=1
    export ECAZ_HARDENING_LOG_ACTIVE
    exec script -q "$log_file" bash "$0" "$lane" "$@"
  fi
fi

case "$lane" in
  cargo-audit)
    need_cmd cargo-audit "cargo install cargo-audit"
    cargo audit
    ;;
  cargo-deny-full)
    need_cmd cargo-deny "cargo install cargo-deny"
    cargo deny check
    ;;
  cargo-vet)
    need_cmd cargo-vet "cargo install cargo-vet"
    if [ ! -f supply-chain/config.toml ]; then
      cat >&2 <<'EOF'
missing cargo-vet setup: supply-chain/config.toml
initialize/report mode:
  cargo vet init
  cargo vet
EOF
      exit 127
    fi
    cargo vet
    ;;
  cargo-geiger)
    need_cmd cargo-geiger "cargo install cargo-geiger"
    geiger_manifest="$PWD/crates/ecaz-cli/Cargo.toml"
    set +e
    cargo geiger --manifest-path "$geiger_manifest"
    status=$?
    set -e
    if [ "$status" -eq 1 ]; then
      echo "cargo-geiger completed with unsafe findings; review the report above."
      exit 0
    fi
    exit "$status"
    ;;
  rudra)
    mkdir -p review/30034-task34-comprehensive-hardening/artifacts
    rudra_manifest="Cargo.toml"
    while [ "$#" -gt 0 ]; do
      case "$1" in
        --manifest-path)
          rudra_manifest="${2:-}"
          if [ -z "$rudra_manifest" ]; then
            echo "missing value for --manifest-path" >&2
            exit 2
          fi
          shift 2
          ;;
        *)
          echo "unknown rudra flag: $1" >&2
          exit 2
          ;;
      esac
    done
    if command -v cargo-rudra >/dev/null 2>&1; then
      (
        cd "$(dirname "$rudra_manifest")"
        cargo rudra
      ) 2>&1 | tee review/30034-task34-comprehensive-hardening/artifacts/rudra.log
    else
      rudra_checkout="$ECAZ_HARDENING_TOOLS_DIR/Rudra"
      rudra_home="$ECAZ_HARDENING_TOOLS_DIR/rudra-home"
      if [ -x "$rudra_checkout/docker-helper/docker-cargo-rudra" ] && [ -d "$rudra_home" ]; then
        PATH="$rudra_checkout/docker-helper:$PATH"
        export PATH
        RUDRA_RUNNER_HOME="$rudra_home"
        export RUDRA_RUNNER_HOME
        CARGO_ARGS=""
        export CARGO_ARGS
        rudra_source="$PWD"
        if [ "$rudra_manifest" != "Cargo.toml" ]; then
          rudra_workspace="$PWD/target/rudra-work-$$"
          mkdir -p "$rudra_workspace"
          tar \
            --exclude .git \
            --exclude target \
            --exclude Cargo.lock \
            -cf - . | tar -C "$rudra_workspace" -xf -
          rudra_source="$rudra_workspace/$(dirname "$rudra_manifest")"
        fi
        script -q review/30034-task34-comprehensive-hardening/artifacts/rudra.log docker-cargo-rudra "$rudra_source"
        cat review/30034-task34-comprehensive-hardening/artifacts/rudra.log
      else
        cat >&2 <<EOF
missing optional hardening tool: cargo-rudra or docker-cargo-rudra
install/setup:
  bash scripts/install_hardening_tools.sh --rudra
EOF
        exit 127
      fi
    fi
    ;;
  mirai)
    need_cmd cargo-mirai "Build MIRAI from https://github.com/endorlabs/MIRAI and ensure cargo-mirai is on PATH; the crates.io mirai package is not the analyzer"
    PATH="$(dirname "$RUSTUP_BIN"):$PATH"
    export PATH
    RUSTUP_TOOLCHAIN=nightly-2025-01-10
    export RUSTUP_TOOLCHAIN
    cargo mirai --manifest-path hardening/careful/Cargo.toml
    ;;
  flux)
    need_cmd cargo-flux "Install Flux from https://flux-rs.github.io/flux/guide/install.html and ensure cargo-flux is on PATH"
    PATH="$(dirname "$RUSTUP_BIN"):$PATH"
    export PATH
    RUSTUP_TOOLCHAIN=nightly-2025-11-25
    export RUSTUP_TOOLCHAIN
    cargo flux --manifest-path hardening/flux/Cargo.toml
    ;;
  miri-expanded)
    need_nightly_miri
    nightly_path
    cargo miri test --lib -- miri_
    ;;
  cargo-careful)
    need_nightly
    need_cmd cargo-careful "cargo install cargo-careful"
    nightly_path
    cargo careful test "${careful_cargo_config[@]}" --manifest-path hardening/careful/Cargo.toml
    ;;
  fuzz-all-short)
    need_nightly
    need_cmd cargo-fuzz "cargo install cargo-fuzz"
    nightly_path
    seconds=30
    while [ "$#" -gt 0 ]; do
      case "$1" in
        --seconds)
          seconds="${2:-}"
          if [ -z "$seconds" ]; then
            echo "missing value for --seconds" >&2
            exit 2
          fi
          shift 2
          ;;
        *)
          echo "unknown fuzz-all-short flag: $1" >&2
          exit 2
          ;;
      esac
    done
    (
      cd fuzz
      for target in \
        fuzz_parse_text \
        fuzz_unpack_mse \
        fuzz_element_tuple_decode \
        fuzz_neighbor_tuple_decode \
        fuzz_diskann_metadata_decode \
        fuzz_item_pointer_decode \
        fuzz_vector_normalize
      do
        cargo fuzz run "$target" -- -max_total_time="$seconds"
      done
    )
    ;;
  afl-decoders)
    need_nightly
    need_cmd cargo-afl "cargo install cargo-afl"
    nightly_path
    cargo afl config --build
    cargo afl build --manifest-path fuzz/Cargo.toml --bin fuzz_diskann_metadata_decode
    cargo afl build --manifest-path fuzz/Cargo.toml --bin fuzz_item_pointer_decode
    ;;
  kani)
    need_cmd cargo-kani "cargo install --locked kani-verifier"
    cargo kani --manifest-path hardening/kani/Cargo.toml --harness kani_item_pointer_decode_contract
    ;;
  loom)
    cargo test --manifest-path hardening/loom/Cargo.toml
    ;;
  shuttle)
    cargo test --manifest-path hardening/shuttle/Cargo.toml
    ;;
  sanitizer-asan)
    run_sanitized_lib_tests address
    ;;
  sanitizer-lsan)
    run_sanitized_lib_tests leak
    ;;
  sanitizer-tsan)
    run_sanitized_lib_tests thread
    ;;
  sanitizer-msan)
    run_sanitized_lib_tests memory
    ;;
  sanitizer-pg18-asan)
    run_sanitized_pg18_tests address
    ;;
  sanitizer-pg18-tsan)
    run_sanitized_pg18_tests thread
    ;;
  sqlsmith-pg18)
    need_cmd sqlsmith "Install SQLsmith and ensure sqlsmith is on PATH"
    dsn="${ECAZ_HARDENING_SQLSMITH_DSN:-}"
    while [ "$#" -gt 0 ]; do
      case "$1" in
        --dsn)
          dsn="${2:-}"
          if [ -z "$dsn" ]; then
            echo "missing value for --dsn" >&2
            exit 2
          fi
          shift 2
          ;;
        *)
          echo "unknown sqlsmith-pg18 flag: $1" >&2
          exit 2
          ;;
      esac
    done
    if [ -z "$dsn" ]; then
      cat >&2 <<'EOF'
missing SQLsmith DSN
setup:
  Start a PG18 cluster with ecaz installed, then pass a libpq connection string:
  make sqlsmith-pg18 SQLSMITH_DSN='postgresql://localhost/postgres'
EOF
      exit 127
    fi
    sqlsmith "$dsn"
    ;;
  *)
    usage >&2
    echo "unknown lane: $lane" >&2
    exit 2
    ;;
esac
