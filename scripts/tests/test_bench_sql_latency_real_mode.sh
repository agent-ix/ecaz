#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd -- "${script_dir}/../.." && pwd)"

tmp_dir="$(mktemp -d)"
trap 'rm -rf "$tmp_dir"' EXIT

fake_psql="${tmp_dir}/fake_psql.sh"
psql_log="${tmp_dir}/psql.log"
stdout_file="${tmp_dir}/stdout.txt"
summary_file="${tmp_dir}/summary.txt"

cat > "$fake_psql" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail

cmd=""
while [[ $# -gt 0 ]]; do
  case "$1" in
    -c)
      cmd="$2"
      shift 2
      ;;
    *)
      shift
      ;;
  esac
done

if [[ -n "$cmd" ]]; then
  case "$cmd" in
    "SHOW shared_buffers;")
      printf '128MB\n'
      exit 0
      ;;
    "SHOW work_mem;")
      printf '4MB\n'
      exit 0
      ;;
    "SHOW max_parallel_workers_per_gather;")
      printf '2\n'
      exit 0
      ;;
    "SELECT count(*) FROM tqhnsw_real_10k_queries;")
      printf '2\n'
      exit 0
      ;;
    "SELECT source FROM tqhnsw_real_10k_queries ORDER BY id LIMIT 2;")
      printf '{1,2}\n{3,4}\n'
      exit 0
      ;;
    "SELECT to_regclass('tqhnsw_real_10k_m8_idx') IS NOT NULL;")
      printf 't\n'
      exit 0
      ;;
  esac
  echo "unexpected -c command: $cmd" >&2
  exit 1
fi

sql="$(cat)"
printf '%s\n---\n' "$sql" >> "${FAKE_PSQL_LOG:?}"

if [[ "$sql" == *"SET tqhnsw.ef_search = 40;"* ]]; then
  printf '[{"Execution Time": 1.0}]\n'
  exit 0
fi

if [[ "$sql" == *"SET tqhnsw.ef_search = 500;"* ]]; then
  printf '[{"Execution Time": 5.0}]\n'
  exit 0
fi

echo "unexpected SQL stdin payload" >&2
exit 1
EOF

chmod +x "$fake_psql"

FAKE_PSQL_LOG="$psql_log" \
TQV_PSQL_BIN="$fake_psql" \
bash "${repo_root}/scripts/bench_sql_latency.sh" \
  --prefix tqhnsw_real_10k \
  --m 8 \
  --ef-search 40,500 \
  --query-limit 2 \
  --output "$summary_file" \
  > "$stdout_file"

grep -q '^shared_buffers: 128MB$' "$stdout_file"
grep -q '^work_mem: 4MB$' "$stdout_file"
grep -q '^max_parallel_workers_per_gather: 2$' "$stdout_file"
grep -q 'cache_state: operator-supplied; script does not warm cache' "$stdout_file"

grep -q 'server_qps=' "$summary_file"
if grep -q ' qps=' "$summary_file"; then
  echo "unexpected legacy qps field in summary output" >&2
  exit 1
fi

grep -q 'ef_search=40' "$summary_file"
grep -q 'server_qps=1000.00' "$summary_file"
grep -q 'ef_search=500' "$summary_file"
grep -q 'server_qps=200.00' "$summary_file"

grep -q 'SET tqhnsw.ef_search = 40;' "$psql_log"
grep -q 'SET tqhnsw.ef_search = 500;' "$psql_log"
if grep -q 'SET LOCAL' "$psql_log"; then
  echo "unexpected SET LOCAL found in real-corpus bench SQL" >&2
  exit 1
fi
