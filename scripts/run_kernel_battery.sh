#!/usr/bin/env bash
# Reproducible kernel-microbench battery for the quant scoring kernels.
#
# Captures, for a single named output directory, the full attribution
# stack reviewers need to reproduce optimization claims on a given host:
#   - criterion wall-time per kernel (target/criterion-* and a tee'd log)
#   - perf-stat hardware counters in non-multiplexed groups (compute A,
#     memory B) plus a top-down microarchitecture run
#   - iai-callgrind exact instruction counts (best-effort; valgrind on
#     aarch64 can be very slow)
#   - dhat heap profiles for encode + score paths
#   - flamegraph SVG of the criterion bench
#   - emitted aarch64/x86_64 assembly for the named hot kernels
#   - host environment snapshot (lscpu, /proc/cpuinfo Features,
#     rustc --version --verbose, /proc/sys/kernel/perf_event_paranoid,
#     /.cargo/config.toml)
#
# Host profile tuning is in this script so external reviewers can
# rerun the same battery on the same host class and get comparable
# numbers. See `docs/benchmarks.md` "Kernel battery" section.

set -euo pipefail

usage() {
  cat <<'EOF'
Usage:
  scripts/run_kernel_battery.sh --out <dir> [--profile small|large|local]
                                [--skip-iai] [--skip-flamegraph]
                                [--skip-asm] [--skip-dhat]
                                [--skip-stream] [--repo-root <path>]

Required:
  --out <dir>          Directory to write all artifacts into. Will be
                       created. Intended layout: review/<packet>/artifacts/<host>/kernels

Options:
  --profile <p>        Host profile. Tunes criterion sample-size and
                       process priority for the target CPU count.
                         small  -- 2 vCPU (m8g.large class). sample-size 30,
                                   warm-up 1s, measurement 2s, nice 10,
                                   30s sleep between perf-stat runs.
                                   This avoids starving the SSM agent.
                         large  -- 8+ vCPU. sample-size 50, defaults
                                   otherwise.
                         local  -- developer workstation. criterion defaults.
                       (default: local)
  --skip-iai           Skip iai-callgrind runs. Use on aarch64 if
                       valgrind is unavailable or too slow.
  --skip-flamegraph    Skip cargo-flamegraph run.
  --skip-asm           Skip cargo-asm disassembly capture.
  --skip-dhat          Skip dhat heap profile.
  --skip-stream        Skip STREAM memory-bandwidth measurement.
  --repo-root <path>   Path to ecaz repo (default: cwd).

Environment:
  CARGO_BIN            Path to cargo (default: cargo on PATH).
  RUSTC_BIN            Path to rustc (default: rustc on PATH).

Example (on m8g.large bench host):
  scripts/run_kernel_battery.sh --out /tmp/artifacts/kernels \
      --profile small --skip-iai
EOF
}

OUT=""
PROFILE="local"
SKIP_IAI=0
SKIP_FLAMEGRAPH=0
SKIP_ASM=0
SKIP_DHAT=0
SKIP_STREAM=0
REPO_ROOT="$(pwd)"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --out) OUT="$2"; shift 2 ;;
    --profile) PROFILE="$2"; shift 2 ;;
    --skip-iai) SKIP_IAI=1; shift ;;
    --skip-flamegraph) SKIP_FLAMEGRAPH=1; shift ;;
    --skip-asm) SKIP_ASM=1; shift ;;
    --skip-dhat) SKIP_DHAT=1; shift ;;
    --skip-stream) SKIP_STREAM=1; shift ;;
    --repo-root) REPO_ROOT="$2"; shift 2 ;;
    -h|--help) usage; exit 0 ;;
    *) echo "unknown arg: $1" >&2; usage; exit 1 ;;
  esac
done

if [[ -z "$OUT" ]]; then
  echo "error: --out is required" >&2
  usage
  exit 1
fi

case "$PROFILE" in
  small)
    # 2-vCPU host (m8g.large class). Pin to a single core so the other
    # vCPU stays available for the OS / SSM agent. Without taskset the
    # SSM agent reliably gets starved during criterion + perf runs and
    # the only recovery is a forced EC2 stop/start.
    CRIT_FLAGS="--sample-size 30 --warm-up-time 1 --measurement-time 2"
    NICE="taskset -c 0 nice -n 10 ionice -c 3 --"
    INTER_RUN_SLEEP=30
    ;;
  large)
    CRIT_FLAGS="--sample-size 50"
    NICE=""
    INTER_RUN_SLEEP=5
    ;;
  local)
    CRIT_FLAGS=""
    NICE=""
    INTER_RUN_SLEEP=0
    ;;
  *)
    echo "error: --profile must be small|large|local" >&2
    exit 1
    ;;
esac

CARGO="${CARGO_BIN:-cargo}"
RUSTC="${RUSTC_BIN:-rustc}"

mkdir -p "$OUT"
cd "$REPO_ROOT"

log() { echo "[kernel-battery] $(date '+%H:%M:%S') $*"; }

# Each step records its exact command at the top of its log so reviewers
# can copy-paste a reproduction without parsing the script.
record_cmd() {
  local logfile="$1"; shift
  {
    echo "# command: $*"
    echo "# date:    $(date -Is)"
    echo "# host:    $(uname -a)"
    echo "# profile: $PROFILE"
    echo "# crit_flags: $CRIT_FLAGS"
    echo "# ---"
  } > "$logfile"
}

# --- 0. Environment snapshot ---
ENV_DIR="$OUT/env"
mkdir -p "$ENV_DIR"
log "capturing host environment to $ENV_DIR"
lscpu > "$ENV_DIR/lscpu.txt" 2>&1 || true
cp /proc/cpuinfo "$ENV_DIR/cpuinfo.txt" 2>/dev/null || true
uname -a > "$ENV_DIR/uname.txt"
cat /etc/os-release > "$ENV_DIR/os-release.txt" 2>/dev/null || true
cat /proc/sys/kernel/perf_event_paranoid > "$ENV_DIR/perf_paranoid.txt" 2>/dev/null || true
cat /sys/devices/system/cpu/cpu0/cpufreq/scaling_governor > "$ENV_DIR/governor.txt" 2>/dev/null || echo "(no cpufreq governor — fixed clock)" > "$ENV_DIR/governor.txt"
"$RUSTC" --version --verbose > "$ENV_DIR/rustc.txt" 2>&1 || true
"$CARGO" --version > "$ENV_DIR/cargo.txt" 2>&1 || true
cp "$REPO_ROOT/.cargo/config.toml" "$ENV_DIR/cargo_config.toml" 2>/dev/null || true

# --- 1. Criterion baseline (wall-time) ---
log "criterion baseline: $CARGO bench --features bench --bench quant_score -- $CRIT_FLAGS"
record_cmd "$OUT/criterion-quant_score.log" "$NICE $CARGO bench --features bench --bench quant_score -- $CRIT_FLAGS"
$NICE "$CARGO" bench --features bench --bench quant_score -- $CRIT_FLAGS 2>&1 \
  | tee -a "$OUT/criterion-quant_score.log"
sleep "$INTER_RUN_SLEEP"

# --- 2. perf-stat Group A (compute + branch) ---
if command -v perf >/dev/null 2>&1; then
  log "perf stat group A (compute+branch)"
  record_cmd "$OUT/perf-stat-quant_score-A.log" \
    "$NICE perf stat -e cycles,instructions,branches,branch-misses,stalled-cycles-frontend,stalled-cycles-backend $CARGO bench --features bench --bench quant_score -- $CRIT_FLAGS"
  $NICE perf stat -e cycles,instructions,branches,branch-misses,stalled-cycles-frontend,stalled-cycles-backend \
    "$CARGO" bench --features bench --bench quant_score -- $CRIT_FLAGS \
    >> "$OUT/perf-stat-quant_score-A.log" 2>&1 || true
  sleep "$INTER_RUN_SLEEP"

  # --- 3. perf-stat Group B (memory hierarchy) ---
  log "perf stat group B (memory)"
  record_cmd "$OUT/perf-stat-quant_score-B.log" \
    "$NICE perf stat -e L1-dcache-loads,L1-dcache-load-misses,LLC-loads,LLC-load-misses,dTLB-load-misses,iTLB-load-misses $CARGO bench --features bench --bench quant_score -- $CRIT_FLAGS"
  $NICE perf stat -e L1-dcache-loads,L1-dcache-load-misses,LLC-loads,LLC-load-misses,dTLB-load-misses,iTLB-load-misses \
    "$CARGO" bench --features bench --bench quant_score -- $CRIT_FLAGS \
    >> "$OUT/perf-stat-quant_score-B.log" 2>&1 || true
  sleep "$INTER_RUN_SLEEP"

  # --- 4. perf-stat top-down ---
  log "perf stat --topdown"
  record_cmd "$OUT/perf-stat-quant_score-topdown.log" \
    "$NICE perf stat --topdown $CARGO bench --features bench --bench quant_score -- $CRIT_FLAGS"
  $NICE perf stat --topdown \
    "$CARGO" bench --features bench --bench quant_score -- $CRIT_FLAGS \
    >> "$OUT/perf-stat-quant_score-topdown.log" 2>&1 || true
  sleep "$INTER_RUN_SLEEP"
else
  log "perf not installed; skipping perf-stat sections"
fi

# --- 5. iai-callgrind ---
if [[ $SKIP_IAI -eq 0 ]] && command -v valgrind >/dev/null 2>&1; then
  log "iai-callgrind (slow on aarch64; --skip-iai to bypass)"
  for bench in iai_quant_score iai_hadamard iai_bitpack; do
    record_cmd "$OUT/iai-${bench}.log" "$CARGO bench --features bench --bench $bench"
    "$CARGO" bench --features bench --bench "$bench" >> "$OUT/iai-${bench}.log" 2>&1 || true
    sleep "$INTER_RUN_SLEEP"
  done
else
  log "skipping iai-callgrind (--skip-iai or valgrind missing)"
fi

# --- 6. flamegraph ---
if [[ $SKIP_FLAMEGRAPH -eq 0 ]] && command -v cargo-flamegraph >/dev/null 2>&1; then
  log "cargo-flamegraph on quant_score"
  record_cmd "$OUT/flame-quant_score.log" "$CARGO flamegraph --features bench --bench quant_score -- --bench"
  $NICE "$CARGO" flamegraph --features bench --bench quant_score -- --bench >> "$OUT/flame-quant_score.log" 2>&1 || true
  if [[ -f flamegraph.svg ]]; then
    mv flamegraph.svg "$OUT/flame-quant_score.svg"
  fi
else
  log "skipping flamegraph"
fi

# --- 7. dhat heap profile ---
if [[ $SKIP_DHAT -eq 0 ]]; then
  log "dhat heap (encode + score paths)"
  for path in encode score; do
    record_cmd "$OUT/dhat-${path}.log" "$CARGO run --release --features bench,dhat-heap --bin dhat_${path}"
    $NICE "$CARGO" run --release --features bench,dhat-heap --bin "dhat_${path}" >> "$OUT/dhat-${path}.log" 2>&1 || true
    if [[ -f dhat-heap.json ]]; then
      mv dhat-heap.json "$OUT/dhat-${path}.json"
    fi
  done
else
  log "skipping dhat"
fi

# --- 8. Disassembly of named hot kernels ---
if [[ $SKIP_ASM -eq 0 ]] && command -v cargo-asm >/dev/null 2>&1; then
  log "capturing asm for hot kernels"
  ASM_DIR="$OUT/asm"
  mkdir -p "$ASM_DIR"
  KERNELS=(
    "ecaz::quant::prod::ProdQuantizer::score_ip_from_parts"
    "ecaz::quant::prod::ProdQuantizer::score_ip_encoded"
    "ecaz::quant::prod::ProdQuantizer::score_ip_codes_lite"
    "ecaz::quant::prod::ProdQuantizer::score_ip_from_split_parts_neon"
    "ecaz::quant::prod::ProdQuantizer::score_ip_mse_codes_neon"
    "ecaz::quant::rabitq::RaBitQQuantizer::estimate_ip"
    "ecaz::quant::hadamard::fwht_in_place"
    "ecaz::quant::hadamard::fwht_in_place_scalar"
  )
  for fn in "${KERNELS[@]}"; do
    short="${fn##*::}"
    "$CARGO" asm --features bench --release "$fn" > "$ASM_DIR/${short}.s" 2>&1 || true
  done
else
  log "skipping cargo-asm (--skip-asm or cargo-show-asm not installed)"
fi

# --- 9. STREAM memory bandwidth ---
if [[ $SKIP_STREAM -eq 0 ]]; then
  STREAM_DIR="$OUT/stream"
  mkdir -p "$STREAM_DIR"
  if [[ ! -x "$STREAM_DIR/stream_c.exe" ]]; then
    log "building STREAM (one-shot, cached in $STREAM_DIR)"
    git clone --depth 1 https://github.com/jeffhammond/STREAM "$STREAM_DIR/src" >> "$STREAM_DIR/build.log" 2>&1 || true
    if [[ -d "$STREAM_DIR/src" ]]; then
      (cd "$STREAM_DIR/src" && make CC=gcc CFLAGS="-O3 -fopenmp" >> "$STREAM_DIR/build.log" 2>&1 || true)
      cp "$STREAM_DIR/src/stream_c.exe" "$STREAM_DIR/" 2>/dev/null || true
    fi
  fi
  if [[ -x "$STREAM_DIR/stream_c.exe" ]]; then
    log "STREAM"
    record_cmd "$STREAM_DIR/stream.log" "$STREAM_DIR/stream_c.exe"
    "$STREAM_DIR/stream_c.exe" >> "$STREAM_DIR/stream.log" 2>&1 || true
  else
    log "STREAM build failed; skipping"
  fi
else
  log "skipping STREAM"
fi

log "kernel battery complete; artifacts in $OUT"
ls -la "$OUT"
