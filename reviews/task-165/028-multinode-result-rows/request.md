# Task 165 packet 028 — structured result rows for the multinode suite step

Coder response to
`reviews/task-165/027-multinode-suite-step/feedback/2026-07-09-01-reviewer.md`.
This packet closes the **P1 "the new suite step emits an empty
`results.jsonl`"** finding at the code level, and records the disposition of the
other two 027 P1s (which require the real multi-instance run).

## P1 (closed here) — empty `results.jsonl`

`parse_result_rows` had no `distann-local-multinode` arm, so the successful
fixture log was discarded by the normalized emitter and the step produced a
zero-byte `results.jsonl`. That violates the repo requirement that every cited
measurement trace to a result row.

**Fix (commit `f98e84775`):** `parse_distann_multinode_rows` scans the
`[distann-multicluster]` fixture log and emits structured `ResultRow`s for the
three decision-grade shapes the fixture prints:

- `RECALL_RESULT n_queries=.. identical=.. mismatched_ids=..` →
  `distinct_recall_identity` rows, plus an `identity_ok` threshold
  (`mismatched_ids == 0`).
- `suite_recall_gate single=.. multi=.. delta=.. pass=..` → `suite_recall_gate`
  rows. The `SKIPPED(...)`/`INCONCLUSIVE(...)` forms (no `single=`) deliberately
  yield no row.
- any `<drill> pass=<bool>` line → `drill_outcome` rows (qual, FR-082, fault
  drills, concurrency, retention, AC-5, disjoint, recovery), so every asserted
  fixture arm traces to a result row.

Two parser unit tests (`distann_multinode_rows_parse_recall_identity_gate_and_drills`,
`distann_multinode_recall_mismatch_sets_identity_not_ok`) pin the extraction and
the mismatch-fails-identity threshold.

## Validation

`cargo test -p ecaz-cli distann_multinode` → `test result: ok. 2 passed`.
Transcript: `artifacts/parser-unit-tests.log`. `cargo build -p ecaz-cli` clean.

## Remaining 027 P1s — disposition (require the real multi-instance run)

These two findings are correct and remain open; they are not code-fixable in
isolation and are folded into the Task 172 real multi-instance run, which this
branch's coder goal now targets:

1. **"Not the claimed real 50k distributed lane" (synthetic 16-dim, 0.08
   absolute recall).** Agreed. The committed 027 config is a synthetic
   result-identity smoke, not a real-corpus quality lane. The real 50k/100k
   distributed distinct-recall must run through the suite against the staged
   real DBpedia corpora (normalized `ecvector`), which is the Task 172
   deliverable. The synthetic config will be relabeled as a smoke there.
2. **"Phase-closeout overstates lifecycle/concurrency."** Agreed that the
   current epoch publish swaps state flags on one graph rather than distinct
   Building/Published record sets, and the concurrency worker asserts session
   success rather than distributed insert/back-edge routing. These are tracked
   with the FR-078 handoff / distributed `aminsert` work (Task 167 P0) and are
   not claimed complete by this packet.

This packet's scope is strictly the result-row emission fix.
