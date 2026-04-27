# Artifact Manifest

Packet: `review/30037-task28-ivf-fullprobe-scorer-alignment`

Head SHA: `1b9ba3e22764c0fb65e66f2f7e0bbcd922fe697e`

Local machine:

- OS: WSL2 Linux `6.6.87.2-microsoft-standard-WSL2`
- CPU: Intel Core i9-10900K, 20 logical CPUs
- PostgreSQL: 18.3, x86_64, gcc 11.4.0
- Storage/cache state: normal local scratch cluster; cache not explicitly
  dropped, so timings are warm/local smoke numbers only.

## Artifacts

### `pg18-ivf-fullprobe-scorer-alignment.sql`

- command artifact: packet-local SQL that materializes exact SQL top-200 and
  IVF full-probe top-200 on the same 20 queries.
- lane / fixture / storage / rerank: IVF, DBPedia anchor 10k x 1536 subset,
  `turboquant`, `rerank = off`.
- isolated surface: reuses packet 30036's copied table and one IVF index.

### `pg18-ivf-fullprobe-scorer-alignment.log`

- command: `cargo run -p ecaz-cli -- dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --port 28818 --file review/30037-task28-ivf-fullprobe-scorer-alignment/artifacts/pg18-ivf-fullprobe-scorer-alignment.sql --raw --log-output review/30037-task28-ivf-fullprobe-scorer-alignment/artifacts/pg18-ivf-fullprobe-scorer-alignment.log`
- timestamp: 2026-04-26 local
- key result lines cited by `request.md`:
  - fixture: `corpus_rows = 10000`, `query_rows = 20`,
    `source_dimensions = 1536`, `index_bytes = 9379840`
  - exact materialization: `SELECT 4000`, `Time: 88093.396 ms (01:28.093)`
  - IVF materialization: `SELECT 4000`, `Time: 5162.813 ms (00:05.163)`
  - top-10 summary: `top10_hits = 184`, `exact_top10_rows = 200`,
    `recall_at_10 = 0.9200`,
    `exact_top10_missing_from_ivf_top200 = 0`,
    `worst_ivf_rank_for_exact_top10 = 14`
  - demotion summary: `demoted_exact_top10_rows = 16`,
    `best_demoted_ivf_rank = 11`, `worst_demoted_ivf_rank = 14`,
    `avg_sql_score_gap = 0.0029758047312498093`,
    `max_sql_score_gap = 0.007695481`
