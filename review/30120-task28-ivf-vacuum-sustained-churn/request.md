# Task 28 IVF Vacuum Sustained Churn

## Scope

This packet records the A3/F2 sustained churn smoke that exposed and then validated `10b141b4` (`ivf: preserve metadata offsets during posting compaction`).

The SQL fixture builds isolated synthetic 50k-row IVF indexes at `nlists=32` and `nlists=64`, then repeats delete/vacuum/refill for three cycles while recording index bytes after each refill.

## Code Fix

Vacuum physical compaction previously compacted posting pages whenever the current list's directory block was not the block being rewritten. That missed mixed pages containing other persistent metadata tuples. If a deleted posting on such a page was compacted, directory or centroid line pointers could be renumbered, invalidating stored metadata TIDs.

`10b141b4` keeps compaction on pure posting pages, but switches mixed pages to no-compact deletion so persistent metadata offsets remain stable.

## Result

| surface | cycle0 build | cycle1 refill | cycle2 refill | cycle3 refill |
|---|---:|---:|---:|---:|
| nlists=32 | 4,464,640 | 4,464,640 | 4,464,640 | 4,464,640 |
| nlists=64 | 4,472,832 | 4,579,328 | 4,751,360 | 4,980,736 |

The nlists=32 surface converges exactly across three refill cycles. The nlists=64 surface no longer crashes, but it still grows over sustained churn.

The nlists=64 refill insert times also show the F2 range-walk concern clearly:

| cycle | n64 refill insert time |
|---|---:|
| 1 | 42,473.263 ms |
| 2 | 124,178.470 ms |
| 3 | 156,093.247 ms |

Raw output is in `artifacts/ivf_sustained_churn_smoke.log`.

## Interpretation

This packet closes the correctness bug discovered while working A3: posting compaction can no longer invalidate metadata tuple offsets on mixed pages.

It does not fully close A3 for all churn shapes. The nlists=64 growth and refill slowdown mean range reuse still needs either better free-space metadata or a different compaction/rewrite strategy before claiming index size tracks live tuple count under sustained churn.

## Validation

- `cargo test -p ecaz --lib posting_delete_compaction_is_disabled_on_mixed_pages`
- `cargo test -p ecaz --lib am::ec_ivf::page::tests`
- `cargo pgrx install --pg-config /home/peter/.pgrx/18.3/pgrx-install/bin/pg_config`
- `cargo run -p ecaz-cli -- --database postgres dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --port 28818 --file review/30120-task28-ivf-vacuum-sustained-churn/artifacts/ivf_sustained_churn_smoke.sql --raw --log-output review/30120-task28-ivf-vacuum-sustained-churn/artifacts/ivf_sustained_churn_smoke.log`
- `cargo pgrx test pg18 test_ec_ivf_vacuum_compacts_deleted_posting_space_for_reuse`
- `cargo pgrx test pg18 test_ec_ivf_vacuum_repairs_empty_list_directory_refs`
- `git diff --check`

## Next

Treat F2 as the next A3 implementation target: the current backward range scan is correct, but the nlists=64 refill timings show it becomes the hot path under fragmentation. A per-list free-space hint or small persisted free-block sidecar is the likely next slice.
