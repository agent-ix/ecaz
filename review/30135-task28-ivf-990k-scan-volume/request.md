# Task 28 IVF 990k Scan Volume Probe

## Scope

This packet uses the existing 990k isolated IVF surface from packet 30130 and the scan-volume counters from packet 30134 to explain the nprobe 32/40/48 latency frontier.

Fixture:

- prefix: `task28_ivf_pqg990k_g8_n128`
- quantizer: `pq_fastscan`
- `pq_group_size=8`
- `nlists=128`
- `rerank=heap_f32`
- `rerank_width=500`
- representative query: first query by `id`
- cache state: warm local PG18; no OS or Postgres cache drop

## Result

Post-fix EXPLAIN run on commit `4426f1ff`:

| nprobe | execution time | selected lists | posting pages read | postings visited | postings scored | pruned by bound | candidates inserted | rerank rows | shared read blocks |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 32 | 788.092 ms | 32 | 5795 | 253879 | 3228 | 250651 | 3228 | 500 | 0 |
| 40 | 893.385 ms | 40 | 7212 | 315958 | 3232 | 312726 | 3232 | 500 | 0 |
| 48 | 1018.736 ms | 48 | 8511 | 372944 | 3235 | 369709 | 3235 | 500 | 0 |

## Interpretation

The nprobe latency arc is dominated by posting-list traversal volume, not by more scored candidates. From nprobe 32 to 48, postings visited increase by about 119k and posting pages by 2716, while postings scored only increase from 3228 to 3235 because score-bound pruning rejects most later postings.

This supports carrying `nprobe=40` as the balanced 990k point from packet 30133: it pays about 62k more visited postings than nprobe 32 for better recall, while nprobe 48 adds another 57k visited postings for the small recall gain already measured.

The first EXPLAIN run in this packet exposed that `Rerank Rows` stayed at zero even when heap rerank ran. Commit `4426f1ff` fixes that counter and makes scan-time rerank obey current relation options; the post-fix run shows `Rerank Rows = 500` for each point.

## Validation

- `cargo run -p ecaz-cli -- --database postgres dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --port 28818 --file review/30135-task28-ivf-990k-scan-volume/artifacts/explain_scan_volume_990k_nprobe32_40_48.sql --raw --log-output review/30135-task28-ivf-990k-scan-volume/artifacts/explain_scan_volume_990k_nprobe32_40_48_after_rerank_fix.log`

Related code validation for commit `4426f1ff`:

- `cargo pgrx test pg18 test_pg18_explain_option_emits_ecaz_stats_group_for_ec_ivf`
- `cargo pgrx test pg18 test_ec_ivf_scan_rerank_uses_current_reloptions`
- `cargo pgrx test pg18 test_ec_ivf_heap_f32_rerank_width_bounds_exact_frontier`
- `cargo test ivf_explain --no-default-features --features pg18`
- `git diff --check`

## Artifacts

- `artifacts/explain_scan_volume_990k_nprobe32_40_48.sql`
- `artifacts/explain_scan_volume_990k_nprobe32_40_48_after_rerank_fix.log`
- `artifacts/explain_scan_volume_990k_nprobe32_40_48.log`
- `artifacts/admin_snapshot_990k.log`
- `artifacts/manifest.md`
