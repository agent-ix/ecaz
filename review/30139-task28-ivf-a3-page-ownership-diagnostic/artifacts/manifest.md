# Artifact Manifest

Head SHA: `067019884ab61a77b044e91a816c7ad092c7df69`

Packet: `review/30139-task28-ivf-a3-page-ownership-diagnostic`

Environment:

- timestamp: `2026-04-28T21:14:41-07:00`
- OS: `Linux DESKTOP-BMB4AFO 6.6.87.2-microsoft-standard-WSL2 x86_64`
- CPU: `Intel(R) Core(TM) i9-10900K CPU @ 3.70GHz`, 20 logical CPUs
- memory: 62 GiB total
- PostgreSQL: 18.3
- surface isolation: isolated same-slice churn tables

## `ivf_same_slice_churn_current.log`

- lane: IVF A3 same-slice churn rerun
- fixture: 50k rows, 4D, `quantizer='turboquant'`, `nlists in {32,64}`, three delete/vacuum/refill cycles
- command: `cargo run -p ecaz-cli -- --database postgres dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --port 28818 --file review/30124-task28-ivf-vacuum-same-slice-churn/artifacts/ivf_same_slice_churn_smoke.sql --raw --log-output review/30139-task28-ivf-a3-page-ownership-diagnostic/artifacts/ivf_same_slice_churn_current.log`
- key result:
  - n32: `cycle0_build=4464640`, `cycle1_refill=4464640`, `cycle2_refill=4464640`, `cycle3_refill=4464640`
  - n64: `cycle0_build=4472832`, `cycle1_refill=4472832`, `cycle2_refill=4489216`, `cycle3_refill=4538368`

## `page_ownership_diagnostic.sql`

- lane: IVF A3 page-ownership diagnostic
- fixture: `task28_ivf_same_slice_n32_idx`, `task28_ivf_same_slice_n64_idx`
- purpose: packet-local SQL that creates the diagnostic function in the existing local database and reports block ownership summaries

## `page_ownership_diagnostic.log`

- lane: IVF A3 page-ownership diagnostic
- fixture: `task28_ivf_same_slice_n32_idx`, `task28_ivf_same_slice_n64_idx`
- command: `cargo run -p ecaz-cli -- --database postgres dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --port 28818 --file review/30139-task28-ivf-a3-page-ownership-diagnostic/artifacts/page_ownership_diagnostic.sql --raw --log-output review/30139-task28-ivf-a3-page-ownership-diagnostic/artifacts/page_ownership_diagnostic.log`
- key result:
  - n32: `posting_blocks=530`, `cross_list_blocks=0`, `mixed_metadata_posting_blocks=1`, `unused_line_pointers=0`, `posting_tuples=50000`, `deleted_posting_tuples=0`
  - n64: `posting_blocks=549`, `cross_list_blocks=21`, `mixed_metadata_posting_blocks=2`, `unused_line_pointers=0`, `posting_tuples=50000`, `deleted_posting_tuples=0`
  - example n64 shared blocks: `517 -> 31,43,61`, `526 -> 31,43,62`, `539 -> 31,43,63`, `545 -> 31,43,63`, `550 -> 16,62,63`
