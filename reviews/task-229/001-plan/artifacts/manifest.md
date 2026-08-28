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

## Reviewer artifacts (seq 02, rereview of request seq-03)

- Artifact: `reviewer-seq02-verification.log`
- Cited by: `feedback/2026-08-26-02-reviewer.md`
- Agent / role / model: Agent IX / reviewer / claude-opus-5
- Head SHA at review: `f70c439e9c07352b18e9cbd3130ef4fd882cfdec`
  (`git diff --name-only 3419c9c75 f70c439e9` touches only `plan/**` and
  `reviews/task-229/**`; `src/`, `sql/`, `crates/`, `spec/`, `tests/` and
  `fixtures/` are unchanged from the `3419c9c75` main the plan targets)
- Task / packet: `reviews/task-229/001-plan/`
- Date: 2026-08-26 America/Los_Angeles
- Lane / fixture / storage format / rerank mode: not applicable — static source
  review, no lane, no fixture, no index built
- Isolated vs shared surfaces: not applicable — no index or table was created
- Command: a single read-only `git` + `grep` + `sed` capture block over
  `src/am/ec_distann/**`, `sql/bootstrap.sql`,
  `crates/ecaz-cli/src/commands/bench/suite.rs`,
  `crates/ecaz-cli/src/commands/dev/distann_multicluster.rs`,
  `plan/tasks/README.md`, and
  `plan/design/ec-distann-recall-latency-roadmap.md`
- Runtime / benchmark / test work: none. No build, PostgreSQL, pgrx, test,
  fixture, or benchmark command was run.
- Key result lines the feedback cites, by log section:
  - W1 — branch tip `f70c439e9`; the diff against `3419c9c75` is `plan/**` and
    `reviews/task-229/**` only.
  - W2 — `payload_projection.rs:34-52`: the shipped
    `PayloadAttributeMask::{Exact, AllColumns}` matches the enum the revised §3
    threads; the seq-01 P2-8 qual contradiction is corrected.
  - W3 — **blocker B1.** `custom_scan.rs:1246` (`Local` skips), `:1256-1261`
    (`Frozen` fetches under `es_snapshot`), `:1264-1285` (`Frozen` refetches
    under `RegisteredSnapshotGuard::latest()`), `:1296-1300` (`Frozen` raises
    `EC_GENERATION_MISSING: published row-tier tuple ({},{}) disappeared`), and
    `:1318` (`RemoteSkipped` is the only arm that skips). The local class errors
    after a two-snapshot retry; it never skips, so §3's single
    `RemoteSkipped`-shaped rule changes local behaviour.
  - W4 — **blocker B2.** `custom_scan.rs:1180-1186` plus the `:1248-1300` arm:
    local `Frozen` hits are resolved per row with one direct
    `table_tuple_fetch_row_version`, no SPI. `remote_endpoint.rs:790,953,1036`
    show the batched `unnest(...) WITH ORDINALITY` shape exists for the remote
    class only. The local lookup mechanism is unstated and the SPI-per-row
    reading would load ~40% of the lazy-10 window (6 remote + 4 local).
  - W5 — `custom_scan.rs:616-640`: `Frozen(ItemPointer)` carries no `vec_id`, so
    §2's `vec_id` echo check is not expressible on the local path as the enum
    stands.
  - W6 — `remote_endpoint.rs:760-776` (`resolve_owned_rows` over shipped
    `vec_ids`) and `generation_read.rs:3552,4612,4638,5996`
    (`benchmark_expanded_locator` is the benchmark-only TID-shipping variant):
    recorded for accuracy; TID keying stays correct remotely and the corruption
    rule stays derivable.
  - W7 — `options.rs:515-541` and `suite.rs:541,4942-5037`: the planned
    `benchmark_covering_sidecar` Userset arm matches the shipped Task 220/221/222
    shape and has an existing isolated-pair validator to extend. P1-4 closed.
  - W8 — `distann_multicluster.rs:1821` (`dm (id bigint, source real[], embedding
    ecvector)`) and `:1904` (`SELECT id FROM dm ORDER BY embedding <#> q.v LIMIT
    k`): cover `'1'` is a valid, exact-subset declaration; the 258-byte bound
    arithmetic (16 × 16 value bytes + 2 null bytes, widest allowed type `uuid`)
    is correct.
  - W9 — every `DISTANN_READY_RECEIPT_BYTES` consumer including
    `lifecycle_wire.rs:132,147`, `sql/bootstrap.sql:355-360` and `:562`; the five
    fixed-triple Rust sites plus `sql/bootstrap.sql:392-394`; and
    `sql/bootstrap.sql:1039,1056,1084` showing the traversal replica has its own
    catalog. P2-4 and P2-5 closed.
  - W10 — `generation_store.rs:317-323,483-486` (the `_ecdz_dir_*` index-name
    precedent, NAMEDATALEN headroom), `manifest_v2.rs:32` and
    `sql/bootstrap.sql:362-364` (34-byte fingerprint, length-only SQL check, no
    hardcoded version byte).
  - W11 — `plan/tasks/README.md:270` and roadmap row 134 both carry the
    post-P1-1 TID-keyed description and the rereview-open state; no stale-header
    defect; entry conditions 1--4 remain satisfied.
- Verdict recorded in the feedback file: **NOT DONE** (2 blockers B1/B2, both
  confined to §3's local-owner paragraph; P1-1..P1-4, P2-1..P2-9 and all eight
  seq-01 question rulings otherwise closed; packet 002 not authorized).

## Coder response to seq 02

- Artifact: `seq02-disposition.md`
- Request revision: seq 04
- Date: 2026-08-26 America/Los_Angeles
- Scope: design-only disposition of B1/B2; remote-only skip scope, local
  two-snapshot/error parity, batched local SPI mechanism, `vec_id` carriage,
  and separately attributable local/remote telemetry
- Runtime / benchmark / test work: none

## Reviewer artifacts (seq 03, narrow B1/B2 rereview of request seq-04)

- Artifact: `reviewer-seq03-verification.log`
- Cited by: `feedback/2026-08-26-03-reviewer.md`
- Agent / role / model: Agent IX / reviewer / claude-opus-5
- Head SHA at review: `3e4e3c2913680f8b2e7cdff04251d4aedfe43cb7`
  (`git diff --name-only 3419c9c75 3e4e3c291` touches only `plan/**` and
  `reviews/task-229/**`, and `git merge-base` of HEAD and `3419c9c75` IS
  `3419c9c75`; `src/`, `sql/`, `crates/`, `spec/`, `tests/` and `fixtures/` are
  unchanged from the main the plan targets)
- Task / packet: `reviews/task-229/001-plan/`
- Date: 2026-08-26 America/Los_Angeles
- Lane / fixture / storage format / rerank mode: not applicable — static source
  review, no lane, no fixture, no index built
- Isolated vs shared surfaces: not applicable — no index or table was created
- Command: a single read-only `git` + `grep` + `sed` capture block over
  `src/am/ec_distann/custom_scan.rs`, `src/storage/snapshot_guard.rs`, and the
  `f70c439e9..3e4e3c291` diff of `request.md` and `seq02-disposition.md`
- Runtime / benchmark / test work: none. No build, PostgreSQL, pgrx, test,
  fixture, or benchmark command was run.
- Scope: deliberately narrow. ONLY seq-02 blockers B1 and B2 were reviewed. No
  seq-01 item that seq-02 closed was reopened or re-examined.
- Key result lines cited by the feedback file:
  - X2 — `git diff f70c439e9 3e4e3c291 -- request.md`: the delta is the §2 local
    paragraph, the §3 remote/local visibility split, the §5 telemetry sentence,
    and the scope statements. §§1, 4, 6 and the packet 003 checkpoint list are
    byte-identical to the seq-03 text seq-02 accepted; §3's structural ERROR
    list is reworded but set-identical. No accepted ruling reopened.
  - X3 — **B1 closed.** `custom_scan.rs:1256-1290` (control: `es_snapshot` then
    `RegisteredSnapshotGuard::latest()`), `:1296-1300` (control errors, never
    skips), `:1318` (`RemoteSkipped` still the only skipping arm) against
    `request.md:143-159`: local rule now stated separately, two-snapshot parity,
    row-tier probe under both snapshots, error string byte-identical to
    `custom_scan.rs:1297`, "They do not skip", and `RemoteSkipped` explicitly
    remote-only.
  - X4 — `snapshot_guard.rs:20-22` and `:60-68`: both `latest()` constructors
    are `RegisterSnapshot(GetLatestSnapshot())`; `ActiveSnapshotGuard` adds only
    the `PushActiveSnapshot` SPI needs, so the plan's substitution claim at
    `request.md:155-157` is accurate. `ActiveSnapshotGuard::latest()` is a real,
    used API (`ec_spire/custom_scan/planner.rs:542`, `ec_hnsw/build.rs:668`).
  - X5 — `request.md:214-216`, `:221-225`, `:229-231`: both tuples share an
    inserting transaction on both creation paths and neither is ever deleted or
    updated, so the local corruption arm is derivable, not asserted.
  - X6 — **B2 closed.** `custom_scan.rs:1422-1436` (window slice already spans
    local and remote outputs), `:1400-1414` (the loop the plan generalizes),
    `:1773-1775` (eager single-batch precedent), `:1733`/`:1789` with
    `hit.vec_id` live at `:1744,:1747,:1792` (the seq-02 W5 `vec_id` gap closes
    by construction) against `request.md:82-89` and `:256-259`: mechanism named
    (batched SPI at `materialize_pending_physical_window`, never per row), site
    feasible, telemetry separates local initial/retry from remote owner work.
  - X7 — `custom_scan.rs:2018-2024`: `custom_scan_recheck` returns `true`
    unconditionally, so `request.md:164-165`'s contract claim is accurate.
- Verdict recorded in the feedback file: **DONE**. Both seq-02 blockers closed;
  **packet 002 implementation authorized**. Three non-gating implementation
  notes carried forward (materialization trigger/guard predicates at `:1213`
  and `:1347-1349` must widen for the new variant; unstated `None` case for
  `ActiveSnapshotGuard::latest()`; eager-mode "proven set" wording nuance).
