# Task 28 IVF Posting Free-Block Hints

## Scope

This packet records `fe2337a3`, a narrow F2/A3 insert-path slice for IVF posting-list range reuse.

The code now:

- records page free space in PostgreSQL's index FSM after posting insert and vacuum rewrite paths,
- consults the FSM before falling back to the full preserved list-range walk,
- keeps a backend-local `(index oid, list_id) -> block` free-block hint so repeated inserts into the same list can retry the last successful reusable block before scanning backward from the tail.

The hint is conservative: it is used only when the hinted block still falls inside the list's current preserved range, and the normal append path still validates page free space under exclusive lock.

## Result

The same sustained churn SQL from packet 30120 was rerun twice:

| variant | n32 cycle3 size | n64 cycle3 size | n32 cycle3 refill | n64 cycle3 refill |
|---|---:|---:|---:|---:|
| 30120 baseline | 4,464,640 | 4,980,736 | 59,626.111 ms | 156,093.247 ms |
| FSM-only trial | 4,464,640 | 4,980,736 | 59,840.705 ms | 156,585.394 ms |
| `fe2337a3` hint | 4,464,640 | 4,997,120 | 45,267.615 ms | 135,267.117 ms |

Interpretation:

- FSM alone does not help this workload.
- The per-list hint reduces refill time in both n32 and n64.
- It does not close A3 physical convergence for n64; n64 still grows under sustained churn.

Raw logs:

- `artifacts/ivf_sustained_churn_fsm_smoke.log`
- `artifacts/ivf_sustained_churn_hint_smoke.log`
- `artifacts/ivf_sustained_churn_hint_smoke.sql`

## Validation

- `cargo test -p ecaz --lib posting_free_hint_roundtrip_is_keyed_by_relation_and_list`
- `cargo test -p ecaz --lib am::ec_ivf::page::tests`
- `cargo pgrx install --pg-config /home/peter/.pgrx/18.3/pgrx-install/bin/pg_config`
- `cargo run -p ecaz-cli -- --database postgres dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --port 28818 --file review/30120-task28-ivf-vacuum-sustained-churn/artifacts/ivf_sustained_churn_smoke.sql --raw --log-output review/30121-task28-ivf-fsm-range-reuse/artifacts/ivf_sustained_churn_hint_smoke.log`
- `cargo pgrx test pg18 test_ec_ivf_vacuum_compacts_deleted_posting_space_for_reuse`
- `cargo pgrx test pg18 test_ec_ivf_vacuum_repairs_empty_list_directory_refs`
- `cargo pgrx test pg18 test_ec_ivf_insert_vacuum_scan_safety`
- `git diff --check`

## Next

F2 is improved but not eliminated. A3 still needs a real physical-convergence design for n64 sustained churn. The current evidence points past backend-local hints toward list-aware free-space metadata or list rewrite compaction that can preserve metadata TIDs while avoiding repeated tail-to-head search.
