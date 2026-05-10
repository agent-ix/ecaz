#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage:
  scripts/run_benchmark_profile.sh <tier> [--dry-run] [ecaz connection args...]

Tiers:
  small     real10k cross-engine + DiskANN prefilter + IVF 25k
  standard  real10k cross-engine + DiskANN prefilter + HNSW 100k + IVF 100k
  full      standard + IVF 25k + IVF 50k checkpoints
  scale     full + IVF 1m fetch/prepare/load lane

Examples:
  scripts/run_benchmark_profile.sh standard --dry-run --database postgres --host /Users/peter/.pgrx --port 28818
  ECAZ_BIN=./target/debug/ecaz scripts/run_benchmark_profile.sh full --database postgres --host /Users/peter/.pgrx --port 28818
  ECAZ_BIN=./target/debug/ecaz scripts/run_benchmark_profile.sh scale --database postgres --host /Users/peter/.pgrx --port 28818
EOF
}

if [[ $# -lt 1 ]]; then
  usage
  exit 1
fi

tier="$1"
shift
dry_run=0
if [[ $# -gt 0 && "$1" == "--dry-run" ]]; then
  dry_run=1
  shift
fi

ecaz_bin="${ECAZ_BIN:-/Users/peter/.cargo/bin/ecaz}"

case "$tier" in
  small)
    suites=(
      "crates/ecaz-cli/suites/profile-cross-engine-real10k.json"
      "crates/ecaz-cli/suites/profile-diskann-prefilter-real10k.json"
      "crates/ecaz-cli/suites/profile-ivf-25k.json"
    )
    ;;
  standard)
    suites=(
      "crates/ecaz-cli/suites/profile-cross-engine-real10k.json"
      "crates/ecaz-cli/suites/profile-diskann-prefilter-real10k.json"
      "crates/ecaz-cli/suites/profile-hnsw-100k.json"
      "crates/ecaz-cli/suites/profile-ivf-100k.json"
    )
    ;;
  full)
    suites=(
      "crates/ecaz-cli/suites/profile-cross-engine-real10k.json"
      "crates/ecaz-cli/suites/profile-diskann-prefilter-real10k.json"
      "crates/ecaz-cli/suites/profile-hnsw-100k.json"
      "crates/ecaz-cli/suites/profile-ivf-25k.json"
      "crates/ecaz-cli/suites/profile-ivf-50k.json"
      "crates/ecaz-cli/suites/profile-ivf-100k.json"
    )
    ;;
  scale)
    suites=(
      "crates/ecaz-cli/suites/profile-cross-engine-real10k.json"
      "crates/ecaz-cli/suites/profile-diskann-prefilter-real10k.json"
      "crates/ecaz-cli/suites/profile-hnsw-100k.json"
      "crates/ecaz-cli/suites/profile-ivf-25k.json"
      "crates/ecaz-cli/suites/profile-ivf-50k.json"
      "crates/ecaz-cli/suites/profile-ivf-100k.json"
      "crates/ecaz-cli/suites/profile-ivf-1m.json"
    )
    ;;
  *)
    echo "unknown tier: $tier" >&2
    usage
    exit 1
    ;;
esac

for suite in "${suites[@]}"; do
  artifact_dir="$(awk -F'"' '/"artifact_dir"/ { print $4; exit }' "$suite")"
  manifest_path="${artifact_dir}/suite-manifest.json"

  echo "[profile:${tier}] auditing ${suite}"
  "$ecaz_bin" "$@" bench suite audit --config "$suite"

  if [[ "$dry_run" -eq 1 ]]; then
    echo "[profile:${tier}] dry-run ${suite}"
    "$ecaz_bin" "$@" bench suite run --config "$suite" --dry-run
  else
    echo "[profile:${tier}] running ${suite}"
    "$ecaz_bin" "$@" bench suite run --config "$suite"
    echo "[profile:${tier}] reporting ${suite}"
    "$ecaz_bin" bench suite report --manifest "$manifest_path"
  fi
done
