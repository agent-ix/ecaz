# Task 28 IVF Vacuum Cursor Reuse

## Scope

This packet records the F2 follow-up that keeps the backend-local posting free-block hint as a descending cursor. When a hinted block can no longer accept a posting tuple, insert advances the hint to the previous block instead of rediscovering that position by repeatedly walking down from the list tail.

This is an incremental range-walk improvement. It does not fully close A3 physical convergence for nlists=64.

## Result

At head `377baf7d`:

- nlists=32 converged through cycle3 at `4,464,640` bytes.
- nlists=64 grew from `4,472,832` bytes at build to `4,964,352` bytes at cycle3.
- nlists=64 cycle3 refill was `134,251.989 ms`.

Comparison:

- Packet 30120 pre-hint baseline: nlists=64 cycle3 `4,980,736` bytes, `156,093.247 ms`.
- Packet 30121 single-block hint: nlists=64 cycle3 `4,997,120` bytes, `135,267.117 ms`.
- Packet 30122 rejected free-block set: nlists=64 cycle3 `5,062,656` bytes, `162,264.328 ms`.

The cursor variant is the best measured local reuse path so far, but the remaining nlists=64 growth means A3 still needs a structural follow-up.

## Validation

- `cargo test -p ecaz --lib posting_free_hint_roundtrip_is_keyed_by_relation_and_list`
- `cargo test -p ecaz --lib am::ec_ivf::page::tests`
- `cargo pgrx install --pg-config /home/peter/.pgrx/18.3/pgrx-install/bin/pg_config`
- `cargo run -p ecaz-cli -- --database postgres dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --port 28818 --file review/30120-task28-ivf-vacuum-sustained-churn/artifacts/ivf_sustained_churn_smoke.sql --raw --log-output review/30123-task28-ivf-vacuum-cursor-reuse/artifacts/ivf_sustained_churn_cursor_smoke.log`
- `cargo pgrx test pg18 test_ec_ivf_vacuum_compacts_deleted_posting_space_for_reuse`
- `cargo pgrx test pg18 test_ec_ivf_vacuum_repairs_empty_list_directory_refs`
- `cargo pgrx test pg18 test_ec_ivf_insert_vacuum_scan_safety`
- `git diff --check -- src/am/ec_ivf/page.rs`

## Artifacts

- `artifacts/ivf_sustained_churn_cursor_smoke.log`
- `artifacts/manifest.md`
