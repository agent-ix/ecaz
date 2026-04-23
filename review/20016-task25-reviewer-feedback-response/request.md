# Review Request: Task 25 Slice 16 — Reviewer Feedback Response

Scope: closes the three items raised by the reviewer in
`review/20014-task25-centered-api/feedback/2026-04-23-01-reviewer.md`.

## Items addressed

### 1. Task-doc / packet contradiction about task-27 gating (Blocking)

- `plan/tasks/25-rabitq-quantizer.md` — replaced the "Pass /
  Marginal / Fail" rubric block with an explicit "superseded for
  Symphony" statement. Records the actual outcomes against each
  operating configuration (absolute 1-bit no-rerank = FAIL at
  10.25 pp; absolute 1-bit rerank K'=100 = PASS; q-bit sweep =
  MARGINAL at q=8; centered path = task-27 territory) and
  explicitly un-defers task 27.
- `plan/tasks/27-symphony-access-method.md` — header flipped from
  "proposed — gated on task 25's recall study passing" to
  "proposed — unblocked by task 25. Start now." Dependency block
  reworded: the centered API landed in task 25 slice 15 and
  replaces the absolute-encoding recall gate as the actual
  prerequisite. The "hard blocker (met)" framing keeps the
  checklist shape intact for anyone auditing the dependency.

Both docs now match the conclusion the slice-13 / slice-14
packets already reached. No inconsistency remains between the
task docs and the packet narrative.

### 2. Stale handoff contract (Blocking)

- `review/20015-task25-task27-handoff-contract-v2/request.md` —
  new packet. Freezes the current public surface (paper-faithful
  estimator from slice 9; q ∈ {1, 2, 4, 8} with three-scalar
  layout from slice 12; seeded `SrhtRotation::with_seed` from
  slice 13; full centered API from slice 15) as the authoritative
  contract task 27 consumes. Includes on-disk layouts for both
  absolute and centered codes, full estimator and bound formulae,
  invariants task 27 may rely on, and an explicit list of what is
  still not frozen.
- `review/20005-task25-task27-handoff-contract/request.md` — header
  stamped `SUPERSEDED` with a pointer to `20015`. Kept in place
  for historical context; reviewer-cited lines (34, 104, 122)
  stay visible so the supersession trail is diff-grep-able.
- `src/lib.rs` `bench_api` — added
  `derive_persisted_sidecar_words`, `persisted_sidecar_word_count`
  (ADR-031 successor path helpers) and `RaBitQScorer` to the
  re-exports. The reviewer specifically flagged these as missing
  from the public surface the contract claimed to freeze; now they
  are exported, and the v2 contract explicitly lists them.

### 3. Measurement-artifact layout convention (Non-blocking)

Per `review/README.md`, measurement logs belong under
`review/{NN}-{topic}/artifacts/` with a sibling `manifest.md` that
records head SHA / lane / command / timestamp / key cited result
lines.

Moved (git mv) raw outputs into the expected layout for all four
measurement packets:

- `review/20007-task25-rabitq-gate-verdict/artifacts/run-dbpedia-10k.txt`
- `review/20009-task25-rabitq-gate-verdict-rerun/artifacts/run-dbpedia-10k-paper-faithful.txt`
- `review/20010-task25-feasibility-rerank/artifacts/run-dbpedia-10k-rerank-100.txt`
- `review/20011-task25-qbit-sweep/artifacts/sweep-dbpedia-10k.txt`

Added `manifest.md` in each packet's `artifacts/` directory with
the required fields. Updated the four `request.md` files to point
at the new `artifacts/...` paths via `sed`; the older at-root
paths are gone.

## Verification

- `cargo test --lib` — 549 pass. 0 regressions.
- `cargo check --lib` — clean.
- `git ls-tree -r --name-only HEAD review/2001*/*.txt` — no raw
  `.txt` logs remain at packet root; all are under
  `artifacts/`.
- The reviewer's three call-out line references still resolve in
  the superseded packet (`review/20005-...:34, 104, 122`) so the
  audit trail is preserved.

## What this slice does NOT do

- No code changes outside `src/lib.rs` bench_api additions.
- No new functional work — the quantizer module stays at slice 15.
- No amendment to ADR-045. The "Open follow-ups" section stays as
  written in slices 14 + 15.

## Closing

Three blocking / non-blocking items from
`review/20014-task25-centered-api/feedback/2026-04-23-01-reviewer.md`
closed. Task 25 is ready for final handoff; task 27 consumes the
v2 contract at `review/20015-task25-task27-handoff-contract-v2/`.
