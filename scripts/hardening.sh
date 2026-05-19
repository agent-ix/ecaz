#!/usr/bin/env bash
set -euo pipefail

export PATH="${HOME}/.cargo/bin:${PATH}"
DEFAULT_CARGO="$(command -v cargo 2>/dev/null || true)"
DEFAULT_RUSTUP="$(command -v rustup 2>/dev/null || true)"
RUSTUP_CARGO="${RUSTUP_CARGO:-${DEFAULT_CARGO:-/opt/homebrew/opt/rustup/bin/cargo}}"
RUSTUP_BIN="${RUSTUP_BIN:-${DEFAULT_RUSTUP:-/opt/homebrew/opt/rustup/bin/rustup}}"
ECAZ_HARDENING_TOOLS_DIR="${ECAZ_HARDENING_TOOLS_DIR:-$HOME/.ecaz/hardening-tools}"

usage() {
  cat <<'EOF'
usage: scripts/hardening.sh <lane>

Local-first hardening lanes. Each lane checks for optional tooling before it
runs and prints install/setup guidance when the tool is missing.

lane flags:
  coverage --output-dir DIR [--html --report-dir DIR]
  mutants --file PATH --output-dir DIR [--jobs N]
  mutants-full --output-dir DIR [--jobs N]
  flake-hunt --seeds N [--fuzz-seconds N] [--output-dir DIR]
  fuzz-all-short --seconds N
  sqlsmith-pg18 --dsn LIBPQ_DSN
  miri-many-seeds uses MIRI_MANY_SEEDS, default 0..128
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

run_miri_prefix() {
  need_nightly_miri
  nightly_path
  cargo miri test --lib -- miri_
}

host_triple() {
  rustc -vV | awk '/host:/ {print $2}'
}

quality_mutation_targets() {
  cat <<'EOF'
src/quant/prod.rs
src/quant/qjl.rs
src/quant/mse.rs
src/quant/simd.rs
src/storage/page.rs
src/am/common/cost.rs
src/am/ec_spire/cost/mod.rs
src/am/ec_spire/coordinator/diagnostics.rs
src/am/ec_diskann/routine.rs
src/am/ec_diskann/scan.rs
src/am/ec_diskann/build.rs
EOF
}

run_coverage_lane() {
  need_cmd cargo-llvm-cov "cargo install cargo-llvm-cov"
  local coverage_cargo=(cargo)
  if [ -x "$RUSTUP_CARGO" ] && "$RUSTUP_CARGO" +stable --version >/dev/null 2>&1; then
    coverage_cargo=("$RUSTUP_CARGO" "+stable")
    if [ -x "$RUSTUP_BIN" ]; then
      "$RUSTUP_BIN" component add llvm-tools-preview >/dev/null
    fi
  fi
  if [ -z "${LLVM_COV:-}" ] && [ -x /opt/homebrew/opt/llvm/bin/llvm-cov ]; then
    LLVM_COV=/opt/homebrew/opt/llvm/bin/llvm-cov
    export LLVM_COV
  fi
  if [ -z "${LLVM_PROFDATA:-}" ] && [ -x /opt/homebrew/opt/llvm/bin/llvm-profdata ]; then
    LLVM_PROFDATA=/opt/homebrew/opt/llvm/bin/llvm-profdata
    export LLVM_PROFDATA
  fi
  local output_dir="target/quality/coverage"
  local report_dir=""
  local html=false
  while [ "$#" -gt 0 ]; do
    case "$1" in
      --output-dir)
        output_dir="${2:-}"
        if [ -z "$output_dir" ]; then
          echo "missing value for --output-dir" >&2
          exit 2
        fi
        shift 2
        ;;
      --html)
        html=true
        shift
        ;;
      --report-dir)
        report_dir="${2:-}"
        if [ -z "$report_dir" ]; then
          echo "missing value for --report-dir" >&2
          exit 2
        fi
        shift 2
        ;;
      *)
        echo "unknown coverage flag: $1" >&2
        exit 2
        ;;
    esac
  done
  mkdir -p "$output_dir"
  local root_summary="$output_dir/root-summary.txt"
  local careful_summary="$output_dir/careful-summary.txt"
  local careful_json="$output_dir/careful-coverage.json"
  "${coverage_cargo[@]}" llvm-cov clean --workspace
  "${coverage_cargo[@]}" llvm-cov --no-report -p ecaz-cli
  "${coverage_cargo[@]}" llvm-cov --no-report --manifest-path hardening/careful/Cargo.toml --lib
  "${coverage_cargo[@]}" llvm-cov report --summary-only > "$root_summary"
  "${coverage_cargo[@]}" llvm-cov report --manifest-path hardening/careful/Cargo.toml --summary-only > "$careful_summary"
  python3 scripts/merge_coverage_summaries.py "$root_summary" "$careful_summary" > "$output_dir/summary.txt"
  "${coverage_cargo[@]}" llvm-cov report --json --output-path "$output_dir/coverage.json"
  "${coverage_cargo[@]}" llvm-cov report --manifest-path hardening/careful/Cargo.toml --json --output-path "$careful_json"
  if [ "$html" = true ]; then
    if [ -z "$report_dir" ]; then
      report_dir="$output_dir/html"
    fi
    "${coverage_cargo[@]}" llvm-cov report --html --output-dir "$report_dir"
  fi
  echo "coverage summary: $output_dir/summary.txt"
  echo "coverage json: $output_dir/coverage.json"
}

run_mutants_lane() {
  need_cmd cargo-mutants "cargo install cargo-mutants"
  local file=""
  local output_dir="target/quality/mutants"
  local jobs="0"
  while [ "$#" -gt 0 ]; do
    case "$1" in
      --file)
        file="${2:-}"
        if [ -z "$file" ]; then
          echo "missing value for --file" >&2
          exit 2
        fi
        shift 2
        ;;
      --output-dir)
        output_dir="${2:-}"
        if [ -z "$output_dir" ]; then
          echo "missing value for --output-dir" >&2
          exit 2
        fi
        shift 2
        ;;
      --jobs)
        jobs="${2:-}"
        if [ -z "$jobs" ]; then
          echo "missing value for --jobs" >&2
          exit 2
        fi
        shift 2
        ;;
      *)
        echo "unknown mutants flag: $1" >&2
        exit 2
        ;;
    esac
  done
  if [ -z "$file" ]; then
    cat >&2 <<'EOF'
missing mutation target
usage:
  make mutants MUTANTS_MODULE=src/quant/prod.rs
EOF
    exit 2
  fi
  mkdir -p "$output_dir"
  local mutate_file="$file"
  local args=(mutants)
  case "$file" in
    src/quant/*|src/storage/page.rs)
      mutate_file="hardening/careful/src/../../../$file"
      args+=(--package ecaz-careful-hardening)
      ;;
  esac
  args+=(--file "$mutate_file" --output "$output_dir/$(basename "$file").mutants")
  if [ "$jobs" != "0" ]; then
    args+=(--jobs "$jobs")
  fi
  cargo "${args[@]}"
}

run_mutants_full_lane() {
  local output_dir="target/quality/mutants"
  local jobs="0"
  while [ "$#" -gt 0 ]; do
    case "$1" in
      --output-dir)
        output_dir="${2:-}"
        if [ -z "$output_dir" ]; then
          echo "missing value for --output-dir" >&2
          exit 2
        fi
        shift 2
        ;;
      --jobs)
        jobs="${2:-}"
        if [ -z "$jobs" ]; then
          echo "missing value for --jobs" >&2
          exit 2
        fi
        shift 2
        ;;
      *)
        echo "unknown mutants-full flag: $1" >&2
        exit 2
        ;;
    esac
  done
  while IFS= read -r target; do
    [ -z "$target" ] && continue
    run_mutants_lane --file "$target" --output-dir "$output_dir" --jobs "$jobs"
  done < <(quality_mutation_targets)
}

run_flake_hunt_lane() {
  local seeds=8
  local fuzz_seconds=10
  local output_dir="target/quality/flake-hunt"
  while [ "$#" -gt 0 ]; do
    case "$1" in
      --seeds)
        seeds="${2:-}"
        if [ -z "$seeds" ]; then
          echo "missing value for --seeds" >&2
          exit 2
        fi
        shift 2
        ;;
      --fuzz-seconds)
        fuzz_seconds="${2:-}"
        if [ -z "$fuzz_seconds" ]; then
          echo "missing value for --fuzz-seconds" >&2
          exit 2
        fi
        shift 2
        ;;
      --output-dir)
        output_dir="${2:-}"
        if [ -z "$output_dir" ]; then
          echo "missing value for --output-dir" >&2
          exit 2
        fi
        shift 2
        ;;
      *)
        echo "unknown flake-hunt flag: $1" >&2
        exit 2
        ;;
    esac
  done
  need_nightly
  need_cmd cargo-fuzz "cargo install cargo-fuzz"
  nightly_path
  mkdir -p "$output_dir"
  output_dir="$(cd "$output_dir" && pwd)"
  {
    printf 'seeds=%s\n' "$seeds"
    printf 'fuzz_seconds=%s\n' "$fuzz_seconds"
    printf 'toolchain=%s\n' "${RUSTUP_TOOLCHAIN:-}"
  } > "$output_dir/manifest.txt"
  local commands_log="$output_dir/expanded-commands.txt"
  : > "$commands_log"
  for seed in $(seq 1 "$seeds"); do
    echo "[flake-hunt] proptest seed=$seed"
    printf 'PROPTEST_RNG_SEED=%s cargo test --features bench --test proptest_quant --test proptest_page -- --test-threads=1\n' "$seed" >> "$commands_log"
    PROPTEST_RNG_SEED="$seed" cargo test --features bench --test proptest_quant --test proptest_page -- --test-threads=1
    echo "[flake-hunt] fuzz seed=$seed seconds=$fuzz_seconds"
    (
      cd fuzz
      for target in fuzz_parse_text fuzz_unpack_mse fuzz_element_tuple_decode fuzz_neighbor_tuple_decode fuzz_diskann_metadata_decode fuzz_item_pointer_decode fuzz_vector_normalize
      do
        printf '(cd fuzz && cargo fuzz run %s -- -seed=%s -max_total_time=%s)\n' "$target" "$seed" "$fuzz_seconds" >> "$commands_log"
        cargo fuzz run "$target" -- -seed="$seed" -max_total_time="$fuzz_seconds"
      done
    )
  done
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
  coverage)
    run_coverage_lane "$@"
    ;;
  mutants)
    run_mutants_lane "$@"
    ;;
  mutants-full)
    run_mutants_full_lane "$@"
    ;;
  flake-hunt)
    run_flake_hunt_lane "$@"
    ;;
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
  mirai)
    need_cmd cargo-mirai "Build MIRAI from https://github.com/endorlabs/MIRAI and ensure cargo-mirai is on PATH; the crates.io mirai package is not the analyzer"
    PATH="$(dirname "$RUSTUP_BIN"):$PATH"
    export PATH
    RUSTUP_TOOLCHAIN=nightly-2025-01-10
    export RUSTUP_TOOLCHAIN
    cargo mirai --manifest-path hardening/careful/Cargo.toml
    ;;
  miri-expanded)
    run_miri_prefix
    ;;
  miri-tree)
    MIRIFLAGS="${MIRIFLAGS:-} -Zmiri-tree-borrows"
    export MIRIFLAGS
    run_miri_prefix
    ;;
  miri-many-seeds)
    MIRIFLAGS="${MIRIFLAGS:-} -Zmiri-many-seeds=${MIRI_MANY_SEEDS:-0..128}"
    export MIRIFLAGS
    run_miri_prefix
    ;;
  miri-full)
    "$0" miri-expanded
    "$0" miri-tree
    "$0" miri-many-seeds
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
