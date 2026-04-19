#!/usr/bin/env bash
set -euo pipefail

usage() {
    cat <<'EOF'
Usage:
  scripts/run_real_corpus_recall_scratch.sh [--db DB] [--socket-dir DIR] [--port PORT] gate \
      --prefix tqhnsw_real_50k \
      --queries-table tqhnsw_real_50k_queries_50 \
      [--storage-format turboquant|pq_fastscan] \
      [--corpus-table tqhnsw_real_50k_corpus] \
      [--detach] \
      [--output-dir /path/to/output]

  scripts/run_real_corpus_recall_scratch.sh [--db DB] [--socket-dir DIR] [--port PORT] summary \
      --m 8 \
      --ef-search 128 \
      --queries-table tqhnsw_real_50k_queries_50 \
      [--prefix tqhnsw_real_50k] \
      [--storage-format turboquant|pq_fastscan] \
      [--index tqhnsw_real_50k_m8_idx] \
      [--corpus-table tqhnsw_real_50k_corpus] \
      [--detach] \
      [--output-dir /path/to/output]

Notes:
  - `gate` runs `tests.tqhnsw_graph_scan_recall_external_gate_report(...)`.
  - `summary` runs `tests.tqhnsw_graph_scan_recall_external_summary(...)`.
  - In detached mode, stdout/stderr are written to files so long-running runs survive
    client-session hiccups.

Defaults:
  output dir: <repo>/tmp/real_corpus_runs
  corpus table for `gate`: <prefix>_corpus
  fixture/index prefix with `--storage-format`: <prefix>_<storage_format>
EOF
}

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd -- "${script_dir}/.." && pwd)"
output_dir="${TQV_REAL_RECALL_OUTPUT_DIR:-${repo_root}/tmp/real_corpus_runs}"
socket_dir=""
port=""
database="${TQV_PG_DATABASE:-postgres}"

subcommand=""
detach=0
prefix=""
corpus_table=""
queries_table=""
index_name=""
m=""
ef_search=""
storage_format=""
fixture_prefix=""

slugify() {
    printf '%s' "$1" | tr -c '[:alnum:]_-' '_'
}

sql_escape_literal() {
    printf '%s' "$1" | sed "s/'/''/g"
}

if [[ $# -eq 0 ]]; then
    usage >&2
    exit 2
fi

while [[ $# -gt 0 ]]; do
    case "$1" in
        --db)
            database="$2"
            shift 2
            ;;
        --socket-dir)
            socket_dir="$2"
            shift 2
            ;;
        --port)
            port="$2"
            shift 2
            ;;
        *)
            break
            ;;
    esac
done

if [[ $# -eq 0 ]]; then
    usage >&2
    exit 2
fi

subcommand="$1"
shift

case "$subcommand" in
    gate|summary)
        ;;
    -h|--help)
        usage
        exit 0
        ;;
    *)
        echo "unknown subcommand: $subcommand" >&2
        usage >&2
        exit 2
        ;;
esac

while [[ $# -gt 0 ]]; do
    case "$1" in
        --prefix)
            prefix="$2"
            shift 2
            ;;
        --corpus-table)
            corpus_table="$2"
            shift 2
            ;;
        --queries-table)
            queries_table="$2"
            shift 2
            ;;
        --index)
            index_name="$2"
            shift 2
            ;;
        --m)
            m="$2"
            shift 2
            ;;
        --ef-search)
            ef_search="$2"
            shift 2
            ;;
        --storage-format)
            storage_format="$2"
            shift 2
            ;;
        --output-dir)
            output_dir="$2"
            shift 2
            ;;
        --detach)
            detach=1
            shift
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

if [[ -z "$queries_table" ]]; then
    echo "--queries-table is required" >&2
    exit 2
fi

if [[ -z "$corpus_table" && -n "$prefix" ]]; then
    corpus_table="${prefix}_corpus"
fi

if [[ -z "$corpus_table" ]]; then
    echo "--corpus-table is required unless --prefix is provided" >&2
    exit 2
fi

fixture_prefix="$prefix"
if [[ -n "$storage_format" ]]; then
    if [[ -z "$prefix" ]]; then
        echo "--storage-format requires --prefix" >&2
        exit 2
    fi
    fixture_prefix="${prefix}_${storage_format}"
fi

select_sql=""
run_sql=""
stem=""
case "$subcommand" in
    gate)
        if [[ -z "$prefix" ]]; then
            echo "--prefix is required for gate runs" >&2
            exit 2
        fi
        select_sql="select * from tests.tqhnsw_graph_scan_recall_external_gate_report('${corpus_table}','${queries_table}','${fixture_prefix}')"
        stem="gate_$(slugify "${fixture_prefix}")_$(slugify "${queries_table}")"
        ;;
    summary)
        if [[ -z "$m" || -z "$ef_search" ]]; then
            echo "--m and --ef-search are required for summary runs" >&2
            exit 2
        fi
        if [[ -z "$index_name" ]]; then
            if [[ -z "$fixture_prefix" ]]; then
                echo "--index is required unless --prefix is provided" >&2
                exit 2
            fi
            index_name="${fixture_prefix}_m${m}_idx"
        fi
        select_sql="select * from tests.tqhnsw_graph_scan_recall_external_summary('${corpus_table}','${queries_table}','${index_name}',${m},${ef_search})"
        stem="summary_$(slugify "${index_name}")_m${m}_ef${ef_search}_$(slugify "${queries_table}")"
        ;;
esac

mkdir -p "$output_dir"

stamp="$(date -u +%Y%m%dT%H%M%SZ)"
sql_file="${output_dir}/${stamp}_${stem}.sql"
out_file="${output_dir}/${stamp}_${stem}.tsv"
log_file="${output_dir}/${stamp}_${stem}.log"

if [[ "$detach" -eq 1 ]]; then
    out_file_sql="$(sql_escape_literal "$out_file")"
    run_sql="copy (${select_sql}) to '${out_file_sql}' with (format text)"
else
    run_sql="${select_sql};"
fi

printf '%s;\n' "$run_sql" >"$sql_file"

cmd=( "${script_dir}/pg17_scratch_psql.sh" )
cmd+=( --db "$database" )
if [[ -n "$socket_dir" ]]; then
    cmd+=( --socket-dir "$socket_dir" )
fi
if [[ -n "$port" ]]; then
    cmd+=( --port "$port" )
fi
cmd+=( --sql "${run_sql};" )

if [[ "$detach" -eq 1 ]]; then
    nohup "${cmd[@]}" >"$out_file" 2>"$log_file" < /dev/null &
    pid="$!"
    printf '[run] started detached %s query\n' "$subcommand"
    printf '[run] pid=%s\n' "$pid"
    printf '[run] sql=%s\n' "$sql_file"
    printf '[run] out=%s\n' "$out_file"
    printf '[run] log=%s\n' "$log_file"
    exit 0
fi

("${cmd[@]}" 2> >(tee "$log_file" >&2)) | tee "$out_file"
printf '[run] sql=%s\n' "$sql_file"
printf '[run] out=%s\n' "$out_file"
printf '[run] log=%s\n' "$log_file"
