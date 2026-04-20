#!/usr/bin/env bash
set -euo pipefail

usage() {
    cat <<'EOF'
Usage:
  scripts/prepare_real_corpus_scratch.sh \
    --profile ec_hnsw_real_50k \
    --parquet /path/to/qdrant-dbpedia-entities-openai3-text-embedding-3-large-1536-1M/data \
    --output-dir /path/to/staged \
    [--storage-format turboquant|pq_fastscan] \
    [--vector-column text-embedding-3-large-1536-embedding] \
    [--id-column _id] \
    [--dim 1536] \
    [--source-dataset "Qdrant dbpedia-entities-openai3-text-embedding-3-large-1536-1M"] \
    [--m 8] [--m 16]

Converts the staged parquet into canonical TSV + manifest files, then loads the
result into the repo-local scratch pg17 cluster through
`scripts/load_real_corpus_scratch.sh`.

Defaults:
  --id-column auto-detect
  --vector-column auto-detect
  --dim 1536
  --m 8
  scratch socket dir: /tmp/tqvector_pgrx_home
  scratch port:       28817
  scratch database:   postgres

Environment overrides:
  PYTHON
  PGRX_HOME
  PGHOST
  PGPORT
  PGDATABASE
  TQV_PSQL_BIN
EOF
}

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd -- "${script_dir}/.." && pwd)"

profile=""
parquet=""
output_dir=""
id_column=""
vector_column=""
dim="1536"
source_dataset=""
storage_format=""
declare -a m_values=()

while [[ $# -gt 0 ]]; do
    case "$1" in
        --profile)
            profile="$2"
            shift 2
            ;;
        --parquet)
            parquet="$2"
            shift 2
            ;;
        --output-dir)
            output_dir="$2"
            shift 2
            ;;
        --id-column)
            id_column="$2"
            shift 2
            ;;
        --vector-column)
            vector_column="$2"
            shift 2
            ;;
        --dim)
            dim="$2"
            shift 2
            ;;
        --source-dataset)
            source_dataset="$2"
            shift 2
            ;;
        --storage-format)
            storage_format="$2"
            shift 2
            ;;
        --m)
            m_values+=("$2")
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

if [[ -z "$profile" || -z "$parquet" || -z "$output_dir" ]]; then
    echo "--profile, --parquet, and --output-dir are required" >&2
    usage >&2
    exit 2
fi

if [[ ${#m_values[@]} -eq 0 ]]; then
    m_values=(8)
fi

if [[ -n "$storage_format" ]]; then
    case "$storage_format" in
        turboquant|pq_fastscan)
            ;;
        *)
            echo "--storage-format must be one of: turboquant, pq_fastscan" >&2
            exit 2
            ;;
    esac
fi

default_python="${repo_root}/../datasets/.venv/bin/python"
if [[ -n "${PYTHON:-}" ]]; then
    python_bin="${PYTHON}"
elif [[ -x "$default_python" ]]; then
    python_bin="$default_python"
else
    python_bin="python3"
fi

converter_args=(
    "${repo_root}/scripts/qdrant_dbpedia_to_tsv.py"
    --profile "$profile"
    --parquet "$parquet"
    --output-dir "$output_dir"
    --dim "$dim"
)
if [[ -n "$id_column" ]]; then
    converter_args+=(--id-column "$id_column")
fi
if [[ -n "$vector_column" ]]; then
    converter_args+=(--vector-column "$vector_column")
fi
if [[ -n "$source_dataset" ]]; then
    converter_args+=(--source-dataset "$source_dataset")
fi

"$python_bin" "${converter_args[@]}"

loader_args=(
    --prefix "$profile"
    --corpus-file "${output_dir}/${profile}_corpus.tsv"
    --queries-file "${output_dir}/${profile}_queries.tsv"
)
for m in "${m_values[@]}"; do
    loader_args+=(--m "$m")
done
if [[ -n "$storage_format" ]]; then
    loader_args+=(--storage-format "$storage_format")
fi

exec "${repo_root}/scripts/load_real_corpus_scratch.sh" "${loader_args[@]}"
