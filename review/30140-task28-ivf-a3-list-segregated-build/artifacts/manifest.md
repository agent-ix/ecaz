# Artifact Manifest

Head SHA: `4e568d22dd7991c8d496598a2be89ee1a7be411e`

Packet: `review/30140-task28-ivf-a3-list-segregated-build`

Environment:

- timestamp: `2026-04-28T21:23:19-07:00`
- OS: `Linux DESKTOP-BMB4AFO 6.6.87.2-microsoft-standard-WSL2 x86_64`
- CPU: `Intel(R) Core(TM) i9-10900K CPU @ 3.70GHz`, 20 logical CPUs
- memory: 62 GiB total
- PostgreSQL: 18.3
- surface isolation: isolated same-slice churn tables

## `ivf_same_slice_churn_list_segregated.log`

- lane: IVF A3 list-segregated build churn smoke
- fixture: 50k rows, 4D, `quantizer='turboquant'`, `nlists in {32,64}`, three delete/vacuum/refill cycles
- command: `cargo run -p ecaz-cli -- --database postgres dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --port 28818 --file review/30124-task28-ivf-vacuum-same-slice-churn/artifacts/ivf_same_slice_churn_smoke.sql --raw --log-output review/30140-task28-ivf-a3-list-segregated-build/artifacts/ivf_same_slice_churn_list_segregated.log`
- key result:
  - n32: `cycle0_build=4603904`, `cycle1_refill=4603904`, `cycle2_refill=4603904`, `cycle3_refill=4603904`
  - n64: `cycle0_build=4734976`, `cycle1_refill=4734976`, `cycle2_refill=4734976`, `cycle3_refill=4734976`

## `page_ownership_list_segregated.log`

- lane: IVF A3 page-ownership diagnostic after list-segregated build
- fixture: `task28_ivf_same_slice_n32_idx`, `task28_ivf_same_slice_n64_idx`
- command: `cargo run -p ecaz-cli -- --database postgres dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --port 28818 --file review/30139-task28-ivf-a3-page-ownership-diagnostic/artifacts/page_ownership_diagnostic.sql --raw --log-output review/30140-task28-ivf-a3-list-segregated-build/artifacts/page_ownership_list_segregated.log`
- key result:
  - n32: `posting_blocks=530`, `cross_list_blocks=0`, `mixed_metadata_posting_blocks=0`, `unused_line_pointers=0`, `posting_tuples=50000`, `deleted_posting_tuples=0`
  - n64: `posting_blocks=550`, `cross_list_blocks=0`, `mixed_metadata_posting_blocks=0`, `unused_line_pointers=0`, `posting_tuples=50000`, `deleted_posting_tuples=0`
