#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
usage: scripts/hardening.sh <lane>

Local-first hardening lanes. Each lane checks for optional tooling before it
runs and prints install/setup guidance when the tool is missing.
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
  need_cmd rustup "Install rustup from https://rustup.rs, then run: rustup toolchain install nightly"
  if ! rustup toolchain list | sed 's/ .*//' | grep -qx nightly; then
    cat >&2 <<'EOF'
missing Rust nightly toolchain
install/setup:
  rustup toolchain install nightly
EOF
    exit 127
  fi
}

need_nightly_miri() {
  need_nightly
  if ! rustup +nightly component list --installed | grep -qx 'miri'; then
    cat >&2 <<'EOF'
missing nightly miri component
install/setup:
  rustup +nightly component add miri
EOF
    exit 127
  fi
}

lane="${1:-}"
if [ -z "$lane" ]; then
  usage >&2
  exit 2
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
    cargo geiger --all-features
    ;;
  rudra)
    need_cmd cargo-rudra "Install Rudra from https://github.com/sslab-gatech/Rudra and ensure cargo-rudra is on PATH"
    mkdir -p review/30034-task34-comprehensive-hardening/artifacts
    cargo rudra | tee review/30034-task34-comprehensive-hardening/artifacts/rudra.log
    ;;
  mirai)
    need_cmd cargo-mirai "cargo install --locked mirai"
    cargo mirai
    ;;
  flux)
    need_cmd flux "Install Flux from https://github.com/flux-rs/flux and ensure flux is on PATH"
    flux check
    ;;
  miri-expanded)
    need_nightly_miri
    cargo +nightly miri test --lib -- miri_
    ;;
  cargo-careful)
    need_cmd cargo-careful "cargo install cargo-careful"
    cargo careful test --lib --tests
    ;;
  fuzz-all-short)
    need_nightly
    need_cmd cargo-fuzz "cargo install cargo-fuzz"
    seconds="${FUZZ_SECONDS:-30}"
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
        cargo +nightly fuzz run "$target" -- -max_total_time="$seconds"
      done
    )
    ;;
  afl-decoders)
    need_nightly
    need_cmd cargo-afl "cargo install afl"
    cargo +nightly afl build --manifest-path fuzz/Cargo.toml --bin fuzz_diskann_metadata_decode
    cargo +nightly afl build --manifest-path fuzz/Cargo.toml --bin fuzz_item_pointer_decode
    ;;
  kani)
    need_cmd cargo-kani "cargo install --locked kani-verifier"
    cargo kani --test kani_item_pointer --harness kani_item_pointer_decode_contract
    ;;
  loom)
    cargo test --manifest-path hardening/loom/Cargo.toml
    ;;
  shuttle)
    cargo test --manifest-path hardening/shuttle/Cargo.toml
    ;;
  sanitizer-asan)
    need_nightly
    RUSTFLAGS="-Zsanitizer=address" cargo +nightly test --lib --target "$(rustc -vV | awk '/host:/ {print $2}')"
    ;;
  sanitizer-lsan)
    need_nightly
    RUSTFLAGS="-Zsanitizer=leak" cargo +nightly test --lib --target "$(rustc -vV | awk '/host:/ {print $2}')"
    ;;
  sanitizer-tsan)
    need_nightly
    RUSTFLAGS="-Zsanitizer=thread" cargo +nightly test --lib --target "$(rustc -vV | awk '/host:/ {print $2}')"
    ;;
  sanitizer-msan)
    need_nightly
    RUSTFLAGS="-Zsanitizer=memory" cargo +nightly test --lib --target "$(rustc -vV | awk '/host:/ {print $2}')"
    ;;
  sanitizer-pg18-asan)
    need_nightly
    RUSTFLAGS="-Zsanitizer=address" cargo +nightly pgrx test pg18
    ;;
  sanitizer-pg18-tsan)
    need_nightly
    RUSTFLAGS="-Zsanitizer=thread" cargo +nightly pgrx test pg18
    ;;
  sqlsmith-pg18)
    need_cmd sqlsmith "Install SQLsmith and ensure sqlsmith is on PATH"
    if [ -z "${ECAZ_HARDENING_SQLSMITH_DSN:-}" ]; then
      cat >&2 <<'EOF'
missing ECAZ_HARDENING_SQLSMITH_DSN
setup:
  Start a PG18 cluster with ecaz installed, then export a libpq connection string, for example:
  export ECAZ_HARDENING_SQLSMITH_DSN='postgresql://localhost/postgres'
EOF
      exit 127
    fi
    sqlsmith "$ECAZ_HARDENING_SQLSMITH_DSN"
    ;;
  *)
    usage >&2
    echo "unknown lane: $lane" >&2
    exit 2
    ;;
esac
