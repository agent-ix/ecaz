# Review Request: bound DiskANN vacuum repair work and service interrupts

Branch: `adr034-diskann-rebased`
Author: coder-2
Target:

- `src/am/ec_diskann/routine.rs`
- `src/am/ec_diskann/scan.rs`
- `review/11081-task17-diskann-vacuum-recall/artifacts/load.log`
- `review/11081-task17-diskann-vacuum-recall/artifacts/pre-vacuum-recall.log`
- `review/11081-task17-diskann-vacuum-recall/artifacts/delete.log`
- `review/11081-task17-diskann-vacuum-recall/artifacts/vacuum.log`
- `review/11081-task17-diskann-vacuum-recall/artifacts/load-fixed.log`
- `review/11081-task17-diskann-vacuum-recall/artifacts/pre-vacuum-recall-fixed.log`
- `review/11081-task17-diskann-vacuum-recall/artifacts/delete-fixed.log`
- `review/11081-task17-diskann-vacuum-recall/artifacts/progress-fixed.log`
- `review/11081-task17-diskann-vacuum-recall/artifacts/vacuum-fixed-cancel.log`
- `review/11081-task17-diskann-vacuum-recall/artifacts/manifest.md`

## What this packet is

This is the next DiskANN AM slice after packet `11080`.

While driving the task-file post-vacuum smoke on the real 10k pg18 fixture,
two DiskANN-specific problems surfaced in the vacuum repair path:

1. vacuum repair reused the full build-time `build_list_size_l` as its
   search/rerank budget for every repaired node, even though the repair pass
   only needs enough candidates to refill at most `R` outgoing slots
2. the long-running DiskANN vacuum / scan loops did not service Postgres
   interrupts, so a runaway backend could sit inside AM code without yielding
   to cancel / terminate

This packet addresses those two runtime issues directly. It does **not** claim
the final post-fix real-corpus recall number yet; the attached `ecaz` logs are
the exploratory operator trail that exposed the problem and will feed the
follow-up smoke rerun on a clean scratch cluster.

## What changed

### `src/am/ec_diskann/routine.rs`

Vacuum repair now derives a dedicated scan budget:

```rust
fn vacuum_repair_scan_budget(build_list_size: usize, graph_degree_r: usize) -> usize {
    build_list_size.min(graph_degree_r.max(1))
}
```

and `plan_vacuum_fill_candidates_for_target(...)` now uses that bounded
budget for all three scan knobs:

```rust
ScanParams {
    entry_point,
    list_size: repair_scan_budget,
    rerank_budget: repair_scan_budget,
    top_k: repair_scan_budget,
}
```

That keeps repair planning proportional to the number of outgoing neighbor
slots a live Vamana node can actually hold, instead of reusing the wider
build-time search budget on every vacuum target.

The same file also adds a DiskANN-local `maybe_check_for_interrupts()` helper
and services interrupts inside the long vacuum loops:

- pass-1 tuple walk over `node_tids`
- pass-2 dead-neighbor unlink walk over `node_tids`
- per-target repair loop in `fill_vacuum_neighbor_slots(...)`
- candidate-consumption loop over `node_results`

Added pure regression coverage:

- `vacuum_repair_scan_budget_caps_at_graph_degree`

### `src/am/ec_diskann/scan.rs`

`greedy_descent_with(...)` now also calls the same DiskANN-local interrupt
helper once per frontier iteration:

```rust
loop {
    maybe_check_for_interrupts();
    ...
}
```

That closes the deepest hot loop used by both ordered scans and vacuum repair
candidate planning, so long-running graph walks no longer monopolize a backend
without checking Postgres interrupts.

The helper is a no-op in the plain Rust unit-test harness and stays active in
real extension / `pg_test` builds, which keeps `cargo test` linkable while the
runtime path still calls `pgrx::check_for_interrupts!()`.

## Operator context

The attached exploratory logs show the path that motivated this slice:

- `artifacts/load.log` — real-10k pg18 load into an isolated DiskANN-only
  scratch database
- `artifacts/pre-vacuum-recall.log` — pre-vacuum baseline at `list_size=128`
  on that isolated database
- `artifacts/delete.log` — deterministic 10% delete (`deleted_rows=1000`)
- `artifacts/vacuum.log` — empty because the first exploratory `VACUUM
  (ANALYZE)` never returned control to the client
- `artifacts/load-fixed.log` — clean-cluster post-fix scratch-db reload using
  the same real 10k fixture and reloptions

The patched operator rerun on a clean scratch database now at least keeps the
same pre-vacuum baseline:

```text
│ 128       ┆ 0.9310   ┆ 0.9966 ┆ 81.90 ms    │
```

and, critically, the patched vacuum path now services cancel requests again.
At `00:02:22.999151` in `artifacts/progress-fixed.log`, the backend was still
inside `vacuuming indexes`; after `pg_cancel_backend(...)`, the client exited
with `artifacts/vacuum-fixed-cancel.log`:

```text
ERROR:  canceling statement due to user request
CONTEXT:  while vacuuming index "ec_hnsw_real_10k_idx" ...
```

That is the runtime behavior this slice set out to restore. The final
post-vacuum Recall@10 smoke remains a follow-up.

## Why this slice

- DiskANN-only and directly in the AM runtime, not CLI work.
- Targets the real vacuum behavior that surfaced under the canonical pg18
  `ecaz` path.
- Keeps the change narrow: bounded repair search plus interrupt servicing, no
  larger vacuum refactor.
- Preserves the existing pg-test repair semantics while cutting obviously
  excess work from the repair scan budget.

## Test evidence

```text
$ cargo test -p ecaz-cli 2>&1 | tail -3

test result: ok. 218 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
```

Also passed on `pg18` for this checkpoint:

- `cargo test`
- `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`

Notable new coverage in that run:

- `am::ec_diskann::routine::tests::vacuum_repair_scan_budget_caps_at_graph_degree`
- `am::ec_diskann::routine::tests::pg_test_ec_diskann_vacuum_refills_broken_neighbor_slot`

## Follow-ups intentionally not in this packet

- The final pg18 real-10k post-vacuum Recall@10 smoke. This packet fixes the
  AM behavior that blocked that run; the measurement rerun is separate.
- Any broader rethink of DiskANN vacuum repair strategy beyond capping the
  search budget at `R`.
- Any changes to build-time interrupt servicing. This slice only touches the
  long-running ordered-scan / vacuum loops that the real operator run hit.
