# Task 229 packet 001 artifact manifest

- Head SHA: `3419c9c758bea7d9940b27d9afbcf9e627e84879`
- Task / packet: `reviews/task-229/001-plan/`
- Date: 2026-08-26 America/Los_Angeles
- Scope: read-only current-main architecture grounding for a planning review
- Runtime/benchmark/test work: none
- Source artifact: `current-main-architecture.md`

The source artifact records the current code surfaces inspected before the
concrete design was written. It contains no benchmark result and makes no
performance claim.

## Reviewer artifacts (seq 01)

- Artifact: `reviewer-seq01-verification.log`
- Cited by: `feedback/2026-08-26-01-reviewer.md`
- Agent / role / model: Agent IX / reviewer / claude-opus-5
- Head SHA at review: `961bb13ee5913fdcff3723baea2e33f637d05203`
  (adds only `reviews/task-229/**` on top of the `3419c9c75` main the plan
  targets; `src/`, `sql/`, `spec/`, `plan/` are unchanged between the two)
- Task / packet: `reviews/task-229/001-plan/`
- Date: 2026-08-26 America/Los_Angeles
- Lane / fixture / storage format / rerank mode: not applicable — static
  source review, no lane, no fixture, no index built
- Isolated vs shared surfaces: not applicable — no index or table was created
- Command: a single read-only `git` + `grep` + `sed` capture block over
  `src/am/ec_distann/**`, `sql/bootstrap.sql`,
  `crates/ecaz-cli/src/commands/bench/suite.rs`,
  `crates/ecaz-cli/src/commands/dev/distann_multicluster.rs`,
  `plan/tasks/222-*.md`, and
  `plan/design/ec-distann-recall-latency-roadmap.md`, redirected to the log
- Runtime / benchmark / test work: none. No build, PostgreSQL, pgrx, test,
  fixture, or benchmark command was run.
- Key result lines the feedback cites, by log section:
  - V2 — `physical_dml.rs:511-529`: a same-identity replacement retains the old
    graph tuple (`SET is_current = false`) and appends a new row-tier tuple at a
    new TID with `record_version + 1`.
  - V3 — `generation_read.rs:2409-2440` and `custom_scan.rs:1248-1270`: row-tier
    payload reads are TID-addressed, therefore version-exact.
  - V4 — `remote_endpoint.rs:908-967` projects `tuple_payload_missing`;
    `custom_scan.rs:1405-1413` turns a missing payload into `RemoteSkipped`, so
    a missing/invisible row-tier tuple is a skip today, not an error.
  - V5 — `options.rs:515-541` registers the Task 220/221/222 same-generation A/B
    GUCs; `suite.rs:4942-5037` validates them as isolated control/candidate
    pairs.
  - V6 — `reloptions` fields exist only on the `spire-local-multinode` step
    (`suite.rs:462-466`); `distann_multicluster.rs:1837,1861,2070` hardcode the
    ec_distann reloption list, so declaring a cover needs new runner fields.
  - V7 — `DistannBuildOptions` already encodes V2/V3 conditional on content and
    decodes V1/V2/V3 under the unchanged containing domain
    (`generation_descriptor.rs:38,60-63,734-736,793-794`).
  - V8 — every `DISTANN_READY_RECEIPT_BYTES = 303` consumer, including
    `lifecycle_wire.rs:132,147`, plus `sql/bootstrap.sql:355-360` and `:562`.
  - V9 — the five sites that enumerate the three generation relation OIDs as a
    fixed tuple, plus `sql/bootstrap.sql:392-394`.
  - V13 — Task 222 and Task 239 entry conditions and the 229 / MAT-27 / ARCH-06
    ledger rows.
- Verdict recorded in the feedback file: **NOT DONE** (4 P1 blockers, 9 P2
  items, rulings on all eight review questions).

## Coder response to seq 01

- Artifact: `seq01-disposition.md`
- Request revision: seq 03
- Date: 2026-08-26 America/Los_Angeles
- Scope: itemized design-only disposition of P1-1..P1-4 and P2-1..P2-9
- Runtime / benchmark / test work: none
