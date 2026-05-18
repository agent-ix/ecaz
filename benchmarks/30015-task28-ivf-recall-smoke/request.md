# Review Request: Task 28 IVF Recall Smoke

Scope: Phase 4 recall-test checkpoint. IVF now has PG coverage for a tiny
full-probe exact oracle and a deterministic corpus smoke comparing exact SQL
scoring, `ec_hnsw`, and full-probe `ec_ivf`.

Task: `plan/tasks/28-ivf-access-method.md` Phase 4

Branch: `task28-ivf`

Head SHA: `d56a23bb00bd7703b461de4092bda05095c22d2d`

Owner: coder2

Files:

- `src/am/ec_ivf/scan.rs`
- `src/lib.rs`
- `plan/tasks/28-ivf-access-method.md`

Validation:

- `cargo check --no-default-features --features pg18 --tests`
- `cargo pgrx test pg18 test_ec_ivf_full_probe_matches_simple_exact_oracle_top1`
- `cargo pgrx test pg18 test_ec_ivf_recall_smoke_compares_exact_hnsw_ivf`
- `cargo pgrx test pg18 test_ec_ivf_gettuple_emits_probe_candidates_with_scores`
- `git diff --check`

Validation notes:

- Validation was PG18-only per the current AGENTS policy and the explicit user
  direction to test with PG18.
- The new PG tests were run against PostgreSQL 18.3 through pgrx.
- No recall-rate measurement claim is made in this packet. The deterministic
  corpus test is a smoke/contract check, not a benchmark gate.

## Summary

This slice closes the Phase 4 recall-tests checklist item:

- Adds an exact SQL top-k helper for `ecvector` query scoring.
- Adds shared helpers to map HNSW and IVF debug scan heap TIDs back to row IDs.
- Adds a tiny full-probe `ec_ivf` oracle test that verifies exact top-1, emits
  all rows, and does not duplicate heap IDs.
- Adds a deterministic 64-row corpus smoke that compares brute force, exact SQL
  scoring, `ec_hnsw`, and full-probe `ec_ivf` top-k overlap.
- Fixes IVF posting-list scan decode to use `code_len` for posting payloads,
  matching the build-side tuple contract where `gamma` is stored separately.
- Updates the task plan to mark Phase 4 recall tests complete and move status to
  Phase 5 live insert.

## Review Focus

Please review for:

- Whether the smoke coverage is the right minimum contract before Phase 5 live
  insert begins.
- Whether the exact SQL helper is the right oracle for `ecvector` rows, given
  IVF v1 is still compressed-only after routing.
- Whether the posting payload decode fix should be reinforced with a pure page
  or scan unit test in addition to the PG scan coverage.
- Whether the deterministic smoke assertions are strong enough without making
  approximate compressed-only scoring flaky.

## Non-Goals

This packet does not implement live insert, vacuum, planner costing, heap/source
rerank, storage-format-specific recall gates, or real-corpus recall/latency
measurement. Phase 8 remains responsible for 10K/50K recall sweeps with
packet-local raw artifacts.
