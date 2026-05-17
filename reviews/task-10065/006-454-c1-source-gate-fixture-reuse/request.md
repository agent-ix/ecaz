# Review Request: C1 Source Gate Fixture Reuse

Current head at execution: `bc93123`

## Context

This checkpoint hardens the direct source-build recall test path without
changing production BUILD behavior.

The branch already has:

- native production BUILD on tqvector-owned HNSW primitives
- real-corpus turboquant gate evidence
- helper-level native builder coverage

What was still awkward was the ignored source-build parity lane in `src/lib.rs`:

- source-backed 10k fixtures rebuilt unconditionally
- the direct source gate test was too broad for routine reruns

That made the remaining merge-readiness evidence path more expensive than it
needed to be.

## What changed

In `src/lib.rs`:

1. `reset_graph_scan_recall_gate_source_fixtures(...)` now reuses an existing
   10k source-fixture corpus and index set via `gate_fixture_already_exists(...)`
   instead of rebuilding on every rerun.

2. Added ignored pg test `test_tqhnsw_graph_scan_recall_source_gate_10k`.
   - exercises the source-build fixture path directly
   - runs the same source-backed gate twice
   - asserts stable results across reruns
   - logs reset timing, rerun timing, and the result vector

3. Narrowed that ignored source-gate test to `25` queries so the lane is
   practical for branch-local parity checks while still covering:
   - source-backed fixture creation
   - source-backed index reuse
   - gate result determinism

No production code changed in this slice.

## Validation

Green checkpoint validation:

```bash
cargo test
bash scripts/run_pgrx_pg17_test.sh
cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings
```

This checkpoint was validated sequentially, so the cited `cargo test` result is
not the invalidated parallel lane issue seen in earlier packets.

## Review focus

1. Is the source-fixture reuse helper an acceptable way to keep the direct
   source-build parity lane cheap enough for routine reruns?
2. Is a 25-query ignored parity surface sufficient as a deterministic branch
   check, with the larger recall claims still anchored in packets `446` and
   `448`?
