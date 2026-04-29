# Artifact Manifest

## Packet

- packet/topic: `30141-task28-ivf-a3-100k-sustained-churn`
- head SHA for committed closure run: `2b72141b`
- lane: Task 28 IVF A3 sustained churn
- surface isolation: isolated one-index-per-table surfaces
- date: 2026-04-28

## `ivf_a3_100k_same_slice_final.log`

- head SHA: `2b72141b`
- lane / fixture / storage format / rerank mode: IVF A3 same-slice sustained
  churn, 100k synthetic 4D rows, `quantizer=turboquant`, `rerank=heap_f32`
- command: `cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 stress ivf-vacuum-scale --table-prefix task28_ivf_a3_100k_same_slice_final --rows 100000 --nlists 32,64 --nprobe 8 --training-sample-rows 10000 --dimensions 4 --vector-period 25000 --quantizer turboquant --cycles 10 --churn-rows 25000 --refill-after-vacuum --same-slice-churn --sample-interval-ms 25 --log-output review/30141-task28-ivf-a3-100k-sustained-churn/artifacts/ivf_a3_100k_same_slice_final.log`
- timestamp: 2026-04-28 22:45-22:48 America/Los_Angeles
- key cited lines:
  - n32 cycle 1: `idx_before 9060352`, `idx_after_refill 9060352`, `hwm_peak_kb 79556`
  - n32 cycle 10: `idx_before 9060352`, `idx_after_refill 9060352`, `hwm_peak_kb 79556`
  - n64 cycle 1: `idx_before 9166848`, `idx_after_refill 9166848`, `hwm_peak_kb 98256`
  - n64 cycle 10: `idx_before 9166848`, `idx_after_refill 9166848`, `hwm_peak_kb 99056`

## `page_ownership_same_slice_final.log`

- head SHA: `2b72141b`
- lane / fixture / storage format / rerank mode: page ownership diagnostic for
  `task28_ivf_a3_100k_same_slice_final_n32_idx` and
  `task28_ivf_a3_100k_same_slice_final_n64_idx`
- command: `cargo run -p ecaz-cli -- --database postgres dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --port 28818 --sql "<page ownership aggregate query>" --raw --log-output review/30141-task28-ivf-a3-100k-sustained-churn/artifacts/page_ownership_same_slice_final.log`
- timestamp: 2026-04-28 22:48 America/Los_Angeles
- key cited lines:
  - `n32 | 1047 | 0 | 0 | 100000 | 100000 | 0 | 0`
  - `n64 | 1061 | 0 | 0 | 100000 | 100000 | 0 | 0`

## `ivf_a3_100k_sustained_churn.log`

- head SHA: `63e3eaf3`
- lane / fixture / storage format / rerank mode: IVF A3 rotating-window
  diagnostic, 100k synthetic 4D rows, `quantizer=turboquant`,
  `rerank=heap_f32`
- command: `cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 stress ivf-vacuum-scale --table-prefix task28_ivf_a3_100k_closure --rows 100000 --nlists 32,64 --nprobe 8 --training-sample-rows 10000 --dimensions 4 --vector-period 100000 --quantizer turboquant --cycles 10 --churn-rows 25000 --refill-after-vacuum --sample-interval-ms 25 --log-output review/30141-task28-ivf-a3-100k-sustained-churn/artifacts/ivf_a3_100k_sustained_churn.log`
- timestamp: 2026-04-28 22:30-22:36 America/Los_Angeles
- key cited lines:
  - n32 cycle 1: `idx_before 9043968`, `idx_after_refill 9043968`
  - n32 cycle 10: `idx_before 11116544`, `idx_after_refill 11198464`
  - n64 cycle 1: `idx_before 9175040`, `idx_after_refill 9175040`
  - n64 cycle 10: `idx_before 11730944`, `idx_after_refill 11829248`

## `page_ownership_after_10cycle.log`

- head SHA: `63e3eaf3`
- lane / fixture / storage format / rerank mode: page ownership diagnostic for
  the rotating-window run
- command: `cargo run -p ecaz-cli -- --database postgres dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --port 28818 --sql "<page ownership aggregate query>" --raw --log-output review/30141-task28-ivf-a3-100k-sustained-churn/artifacts/page_ownership_after_10cycle.log`
- timestamp: 2026-04-28 22:37 America/Los_Angeles
- key cited lines:
  - `n32 | 1066 | 32 | 0 | 100000 | 100000 | 949 | 2`
  - `n64 | 1067 | 24 | 0 | 100000 | 100000 | 952 | 2`

## Exploratory Logs Not Cited As Closure

- `ivf_a3_100k_onecycle_after_insert_fix.log`: one-cycle smoke before stable
  vector-period cleanup; retained as local diagnostic.
- `ivf_a3_100k_onecycle_stable_after_insert_fix.log`: one-cycle smoke while a
  stale backend still pinned dead tuples; retained as diagnostic.
- `ivf_a3_100k_onecycle_stable_clean_after_insert_fix.log`: clean one-cycle
  stable smoke after terminating the stale backend.
- `onecycle_stable_admin_snapshot.log`: failed admin snapshot query with wrong
  signature; not cited.
- `onecycle_stable_manual_vacuum_only.log`: VACUUM VERBOSE proof that the stale
  backend pinned dead tuples.
- `onecycle_stable_manual_vacuum_verbose.log`: failed multi-statement VACUUM
  attempt; not cited.
- `onecycle_stable_page_ownership.log`: diagnostic from the stale-backend run.
- `pg_stat_activity_during_churn.log`: backend state during the interrupted
  first long run.
- `pg_stat_activity_after_not_removable.log`: backend state showing the stale
  backend.
- `terminate_stale_backend.log`: termination of the stale backend.
- `pg_stat_activity_after_10cycle.log`: backend state after the rotating-window
  diagnostic.
- `ivf_a3_100k_same_slice_churn.log`: rotating delete-window run with
  `--vector-period 25000`, retained as diagnostic before `--same-slice-churn`.
