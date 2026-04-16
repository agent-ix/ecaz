#!/usr/bin/env bash
set -euo pipefail

usage() {
    cat <<'EOF'
Usage:
  scripts/restart_adr030_scratch.sh \
      [--window 16] \
      [--grouped-score-mode pq|binary] \
      [--rerank-mode quantized|heap_f32] \
      [--rerank-source-column source_raw] \
      [--exact-scope all|layer0] \
      [--exact-strategy expansion|frontier_head] \
      [--exact-limit N] \
      [--pgrx-home /tmp/tqvector_pgrx_home]

Notes:
  - Always enables the ADR-030 grouped build and scan gates.
  - Enables exact traversal automatically when either `--exact-scope` or
    `--exact-limit` is provided.
  - Restarts the existing scratch postmaster if one is already running.
EOF
}

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd -- "${script_dir}/.." && pwd)"

window="16"
grouped_score_mode="pq"
rerank_mode="quantized"
rerank_source_column=""
exact_enabled=0
exact_scope="all"
exact_strategy="expansion"
exact_limit=""
pgrx_home="${PGRX_HOME:-/tmp/tqvector_pgrx_home}"

while [[ $# -gt 0 ]]; do
    case "$1" in
        --window)
            window="$2"
            shift 2
            ;;
        --grouped-score-mode)
            grouped_score_mode="$2"
            shift 2
            ;;
        --rerank-mode)
            rerank_mode="$2"
            shift 2
            ;;
        --rerank-source-column)
            rerank_source_column="$2"
            shift 2
            ;;
        --exact-scope)
            exact_enabled=1
            exact_scope="$2"
            shift 2
            ;;
        --exact-limit)
            exact_enabled=1
            exact_limit="$2"
            shift 2
            ;;
        --exact-strategy)
            exact_enabled=1
            exact_strategy="$2"
            shift 2
            ;;
        --pgrx-home)
            pgrx_home="$2"
            shift 2
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            echo "unknown argument: $1" >&2
            usage >&2
            exit 2
            ;;
    esac
done

data_dir="${pgrx_home}/data-17"
pid_file="${data_dir}/postmaster.pid"

if [[ -f "${pid_file}" ]]; then
    pid="$(head -n 1 "${pid_file}")"
    if [[ -n "${pid}" ]] && kill -0 "${pid}" 2>/dev/null; then
        printf '[scratch] stopping existing postmaster pid=%s\n' "${pid}"
        kill "${pid}"
        for _ in $(seq 1 30); do
            if ! kill -0 "${pid}" 2>/dev/null; then
                break
            fi
            sleep 1
        done
    fi
fi

printf '[scratch] repo=%s\n' "${repo_root}"
printf '[scratch] pgrx_home=%s\n' "${pgrx_home}"
printf '[scratch] window=%s\n' "${window}"
printf '[scratch] grouped_score_mode=%s\n' "${grouped_score_mode}"
printf '[scratch] rerank_mode=%s\n' "${rerank_mode}"
if [[ -n "${rerank_source_column}" ]]; then
    printf '[scratch] rerank_source_column=%s\n' "${rerank_source_column}"
else
    printf '[scratch] rerank_source_column=build_source_column\n'
fi
if [[ "${exact_enabled}" -eq 1 ]]; then
    printf '[scratch] exact_scope=%s\n' "${exact_scope}"
    printf '[scratch] exact_strategy=%s\n' "${exact_strategy}"
    if [[ -n "${exact_limit}" ]]; then
        printf '[scratch] exact_limit=%s\n' "${exact_limit}"
    else
        printf '[scratch] exact_limit=all\n'
    fi
else
    printf '[scratch] exact_scope=disabled\n'
fi

export TQVECTOR_EXPERIMENTAL_ADR030_V2_BUILD=1
export TQVECTOR_EXPERIMENTAL_ADR030_V2_SCAN=1
export TQVECTOR_EXPERIMENTAL_ADR030_V2_SCAN_WINDOW="${window}"
export TQVECTOR_EXPERIMENTAL_ADR030_V2_SCAN_GROUPED_SCORE_MODE="${grouped_score_mode}"
export TQVECTOR_EXPERIMENTAL_ADR030_V2_SCAN_RERANK_MODE="${rerank_mode}"
if [[ -n "${rerank_source_column}" ]]; then
    export TQVECTOR_EXPERIMENTAL_ADR030_V2_SCAN_RERANK_SOURCE_COLUMN="${rerank_source_column}"
else
    unset TQVECTOR_EXPERIMENTAL_ADR030_V2_SCAN_RERANK_SOURCE_COLUMN 2>/dev/null || true
fi

if [[ "${exact_enabled}" -eq 1 ]]; then
    export TQVECTOR_EXPERIMENTAL_ADR030_V2_SCAN_EXACT_TRAVERSAL=1
    export TQVECTOR_EXPERIMENTAL_ADR030_V2_SCAN_EXACT_TRAVERSAL_SCOPE="${exact_scope}"
    export TQVECTOR_EXPERIMENTAL_ADR030_V2_SCAN_EXACT_TRAVERSAL_STRATEGY="${exact_strategy}"
    if [[ -n "${exact_limit}" ]]; then
        export TQVECTOR_EXPERIMENTAL_ADR030_V2_SCAN_EXACT_TRAVERSAL_LIMIT="${exact_limit}"
    else
        unset TQVECTOR_EXPERIMENTAL_ADR030_V2_SCAN_EXACT_TRAVERSAL_LIMIT 2>/dev/null || true
    fi
else
    unset TQVECTOR_EXPERIMENTAL_ADR030_V2_SCAN_EXACT_TRAVERSAL 2>/dev/null || true
    unset TQVECTOR_EXPERIMENTAL_ADR030_V2_SCAN_EXACT_TRAVERSAL_SCOPE 2>/dev/null || true
    unset TQVECTOR_EXPERIMENTAL_ADR030_V2_SCAN_EXACT_TRAVERSAL_STRATEGY 2>/dev/null || true
    unset TQVECTOR_EXPERIMENTAL_ADR030_V2_SCAN_EXACT_TRAVERSAL_LIMIT 2>/dev/null || true
fi

export PGRX_HOME="${pgrx_home}"

cd "${repo_root}"
exec cargo pgrx start pg17
