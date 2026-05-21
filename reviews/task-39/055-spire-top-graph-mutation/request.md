# Task 39 / 055 — SPIRE top_graph.rs mutation campaign (full verification)

## Goal

Tenth slice of the reviewer-prescribed SPIRE storage mutation
cascade. Drive every mutation in
`src/am/ec_spire/storage/top_graph.rs` to **0 missed / 0 timeouts**.

## Result

**62 mutations enumerated → 62 KILLED, 0 MISSED, 0 timeouts.**

Full per-mutation verification under
`CARGO_TARGET_DIR=$(pwd)/target-mutants` (per reviewer direction
across 050/053/054/055 feedback). 59 KILLED by the existing test
surface; 3 initially MISSED on validate/decode boundary checks,
killed after adding 3 new tests (one decode prefix-only boundary,
one `alpha == 1.0` boundary, one `neighbors.len() == graph_degree`
boundary).

## Methodology

Full per-mutation apply/test/revert via
`/tmp/run_spire_mutations_v2.py` with isolated build cache.

## Code change

- `src/am/ec_spire/storage/tests/top_graph.rs`: added 3 new
  boundary-killing tests. See `triage.md` for the full table.

Source `top_graph.rs` unchanged.

## Validation

Artifacts under `reviews/task-39/055-spire-top-graph-mutation/artifacts/`:

- `top-graph-mutants-enumerated.txt` — full 62 enumeration.
- `manual-verification.log` — 62/62 per-mutation verdicts.
- `post-verification-tests.log` — clean re-run after revert.

`triage.md` documents the killing-test rationale and mutant
re-verification after the tests were added.
