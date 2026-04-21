# 11081 DiskANN vacuum repair runtime artifacts

- Head SHA: `WORKTREE`
- Packet: `11081-task17-diskann-vacuum-recall`
- Lane: `adr034-diskann-rebased`
- Fixture: `ec_hnsw_real_10k`
- Surface: pg18 scratch cluster via `ecaz` + `psql`
- Index profile: `ec_diskann`
- Reloptions: `graph_degree=32`, `build_list_size=100`, `alpha=1.2`
- Table model: shared-table real corpus (`ec_hnsw_real_10k_corpus`, `ec_hnsw_real_10k_queries`)

## `load.log`

- Command:
  `cargo run -p ecaz-cli -- --host /home/peter/.pgrx --port 28818 --database diskann_vacuum_smoke --log-file review/11081-task17-diskann-vacuum-recall/artifacts/load.log corpus load --prefix ec_hnsw_real_10k --corpus-file /home/peter/dev/tqvector/target/real-corpus/ec_hnsw_real_10k/ec_hnsw_real_10k_corpus.tsv --queries-file /home/peter/dev/tqvector/target/real-corpus/ec_hnsw_real_10k/ec_hnsw_real_10k_queries.tsv --profile ec_diskann --reloption graph_degree=32 --reloption build_list_size=100 --reloption alpha=1.2`
- Key lines:
  - `[loader] building ec_hnsw_real_10k_idx using ec_diskann ...`
  - `│ corpus  ┆ ec_hnsw_real_10k_corpus (10000 rows) │`

## `pre-vacuum-recall.log`

- Command:
  `cargo run -p ecaz-cli -- --host /home/peter/.pgrx --port 28818 --database diskann_vacuum_smoke --log-file review/11081-task17-diskann-vacuum-recall/artifacts/pre-vacuum-recall.log bench recall --prefix ec_hnsw_real_10k --profile ec_diskann --k 10 --sweep 128`
- Key lines:
  - `[recall] ground truth in 4.46s`
  - `│ 128       ┆ 0.9310   ┆ 0.9966 ┆ 82.34 ms    │`

## `delete.log`

- Command:
  `psql -h /home/peter/.pgrx -p 28818 -d diskann_vacuum_smoke -Atc "with deleted as (delete from ec_hnsw_real_10k_corpus where id % 10 = 0 returning 1) select 'deleted_rows=' || count(*) from deleted" -o review/11081-task17-diskann-vacuum-recall/artifacts/delete.log`
- Key line:
  - `deleted_rows=1000`

## `load-fixed.log`

- Command:
  `cargo run -p ecaz-cli -- --host /home/peter/.pgrx --port 28818 --database diskann_vacuum_smoke_b --log-file review/11081-task17-diskann-vacuum-recall/artifacts/load-fixed.log corpus load --prefix ec_hnsw_real_10k --corpus-file /home/peter/dev/tqvector/target/real-corpus/ec_hnsw_real_10k/ec_hnsw_real_10k_corpus.tsv --queries-file /home/peter/dev/tqvector/target/real-corpus/ec_hnsw_real_10k/ec_hnsw_real_10k_queries.tsv --profile ec_diskann --reloption graph_degree=32 --reloption build_list_size=100 --reloption alpha=1.2`
- Key lines:
  - `[loader] building ec_hnsw_real_10k_idx using ec_diskann ...`
  - `│ corpus  ┆ ec_hnsw_real_10k_corpus (10000 rows) │`

## `pre-vacuum-recall-fixed.log`

- Command:
  `cargo run -p ecaz-cli -- --host /home/peter/.pgrx --port 28818 --database diskann_vacuum_smoke_b --log-file review/11081-task17-diskann-vacuum-recall/artifacts/pre-vacuum-recall-fixed.log bench recall --prefix ec_hnsw_real_10k --profile ec_diskann --k 10 --sweep 128`
- Key lines:
  - `[recall] ground truth in 4.41s`
  - `│ 128       ┆ 0.9310   ┆ 0.9966 ┆ 81.90 ms    │`

## `delete-fixed.log`

- Command:
  `psql -h /home/peter/.pgrx -p 28818 -d diskann_vacuum_smoke_b -Atc "with deleted as (delete from ec_hnsw_real_10k_corpus where id % 10 = 0 returning 1) select 'deleted_rows=' || count(*) from deleted" -o review/11081-task17-diskann-vacuum-recall/artifacts/delete-fixed.log`
- Key line:
  - `deleted_rows=1000`

## `progress-fixed.log`

- Command:
  `psql -h /home/peter/.pgrx -p 28818 -d diskann_vacuum_smoke_b -Atc "select a.pid, v.phase, clock_timestamp() - a.query_start from pg_stat_activity a join pg_stat_progress_vacuum v using (pid) where a.datname = 'diskann_vacuum_smoke_b' and v.relid = 'ec_hnsw_real_10k_corpus'::regclass"`
- Key line:
  - `1492673|vacuuming indexes|00:02:22.999151`

## `vacuum-fixed-cancel.log`

- Command sequence:
  1. `psql -h /home/peter/.pgrx -p 28818 -d diskann_vacuum_smoke_b -c "vacuum (analyze) ec_hnsw_real_10k_corpus" -o review/11081-task17-diskann-vacuum-recall/artifacts/vacuum-fixed.log`
  2. `psql -h /home/peter/.pgrx -p 28818 -d postgres -Atc "select pg_cancel_backend(1492673)"`
- Key lines:
  - `ERROR:  canceling statement due to user request`
  - `CONTEXT:  while vacuuming index "ec_hnsw_real_10k_idx" of relation "public.ec_hnsw_real_10k_corpus"`

## `vacuum.log` / `vacuum-fixed.log`

- Both files are empty because `psql -o` only captures stdout; the first run never returned control before the backend was force-killed, and the patched run exited on stderr after `pg_cancel_backend(...)`.
