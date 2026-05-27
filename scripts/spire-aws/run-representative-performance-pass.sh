#!/usr/bin/env bash
# Standard entrypoint for the Phase 13e representative performance AWS pass.

set -euo pipefail

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd -- "$script_dir/../.." && pwd)"
aws_dir="$repo_root/infra/spire-aws"

artifact_dir=""
execute=0
allow_preexisting_residue=1
run_preflight=1
reuse_artifact_dir=0

usage() {
  cat <<'EOF'
usage: run-representative-performance-pass.sh --artifact-dir reviews/task-30/<packet>/artifacts [--execute] [--strict-residue] [--skip-preflight]

Dry-run by default. This script standardizes the Phase 13e representative AWS
performance pass without provisioning unless --execute is present.

Options:
  --artifact-dir DIR    Required packet-local artifact directory.
  --execute             Actually run make pass-representative-performance.
  --strict-residue      Do not set SPIRE_AWS_ALLOW_PREEXISTING_RESIDUE=1.
  --skip-preflight      Skip read-only local/AWS preflight checks before execute.
  --reuse-artifact-dir  Allow execute to reuse a directory with prior pass output.

The execute path sets SPIRE_AWS_CONFIRM_PROVISION=yes and runs:

  make -C infra/spire-aws ARTIFACT_DIR=<DIR> pass-representative-performance
EOF
}

die() {
  printf 'ERROR: %s\n' "$*" >&2
  exit 2
}

while (($#)); do
  case "$1" in
    --artifact-dir)
      artifact_dir="${2:?missing value for --artifact-dir}"
      shift 2
      ;;
    --execute)
      execute=1
      shift
      ;;
    --strict-residue)
      allow_preexisting_residue=0
      shift
      ;;
    --skip-preflight)
      run_preflight=0
      shift
      ;;
    --reuse-artifact-dir)
      reuse_artifact_dir=1
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      die "unknown argument: $1"
      ;;
  esac
done

[[ -n "$artifact_dir" ]] || die "--artifact-dir is required"

case "$artifact_dir" in
  reviews/task-30/*/artifacts|"$repo_root"/reviews/task-30/*/artifacts)
    ;;
  *)
    die "--artifact-dir must be packet-local under reviews/task-30/<packet>/artifacts"
    ;;
esac

if [[ "$artifact_dir" != /* ]]; then
  artifact_dir="$repo_root/$artifact_dir"
fi

printf 'SPIRE representative performance pass\n'
printf '  artifact_dir=%s\n' "$artifact_dir"
printf '  execute=%s\n' "$execute"
printf '  allow_preexisting_residue=%s\n' "$allow_preexisting_residue"
printf '  reuse_artifact_dir=%s\n' "$reuse_artifact_dir"

if ((run_preflight)); then
  preflight_env=()
  if ((allow_preexisting_residue)); then
    preflight_env+=(SPIRE_AWS_ALLOW_PREEXISTING_RESIDUE=1)
  fi
  env "${preflight_env[@]}" \
    make -C "$aws_dir" \
      ARTIFACT_DIR="$artifact_dir" \
      preflight-operator preflight-state preflight-permissions preflight-representative-performance
fi

printf 'Command:\n'
if ((allow_preexisting_residue)); then
  printf '  SPIRE_AWS_ALLOW_PREEXISTING_RESIDUE=1 \\\n'
fi
printf '  SPIRE_AWS_CONFIRM_PROVISION=yes \\\n'
printf '  make -C %q ARTIFACT_DIR=%q pass-representative-performance\n' "$aws_dir" "$artifact_dir"

if ((execute == 0)); then
  printf 'Dry run only. Re-run with --execute after explicit AWS approval.\n'
  exit 0
fi

mkdir -p "$artifact_dir"
if ((reuse_artifact_dir == 0)); then
  existing_artifact="$(
    find "$artifact_dir" -maxdepth 1 \
      \( \
        -name 'aws-topology*.json' -o \
        -name 'suite-results-representative*.jsonl' -o \
        -name 'suite-manifest-representative*.json' -o \
        -name 'suite-representative*.json' -o \
        -name 'representative-*.tsv' -o \
        -name '.representative-performance-pass.started' \
      \) \
      -print -quit
  )"
  if [[ -n "$existing_artifact" ]]; then
    die "refusing to reuse artifact directory with prior representative output: $existing_artifact"
  fi

  marker="$artifact_dir/.representative-performance-pass.started"
  if ! (set -o noclobber; printf 'started_at=%s\n' "$(date -Iseconds)" > "$marker") 2>/dev/null; then
    die "refusing to reuse artifact directory with existing representative pass marker: $marker"
  fi
fi

execute_env=(SPIRE_AWS_CONFIRM_PROVISION=yes)
if ((allow_preexisting_residue)); then
  execute_env+=(SPIRE_AWS_ALLOW_PREEXISTING_RESIDUE=1)
fi

env "${execute_env[@]}" \
  make -C "$aws_dir" \
    ARTIFACT_DIR="$artifact_dir" \
    pass-representative-performance
