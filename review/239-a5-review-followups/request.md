# Review Request: A5 Reviewer Follow-Ups

## Context

Branch:
- `main`

Reviewer inputs addressed here:
- `review/229-a5-insert-level-allocation-promotion/feedback/2026-04-11-01-reviewer.md`
- `review/230-a5-forward-links-on-new-node/feedback/2026-04-11-01-reviewer.md`

Task / spec inputs touched:
- `plan/tasks/06-graph-insert.md`
- `spec/functional/FR-016-hnsw-insert.md`
- `spec/adr/ADR-007-query-scoring-and-payload.md`

This is a narrow cleanup checkpoint after A5 closed on `main`. It does **not**
change insert behavior. It only lands the durable comments, invariant checks,
and doc clarifications that the outside review on packets 229 and 230 asked us
to record before more runtime work stacks on top.

Checkpoint scope:

1. clarify the insert-level sampler and metadata-promotion invariants in code
2. make the `ef_construction >= 1` assumption explicit in the insert helper
3. document the insert/build vs scan scoring split in task/spec/ADR surfaces
4. record the `INVALID` placeholder-tid traversal guarantee and the heap-TID
   level-stability caveat in the A5 task doc

## Scope

- `src/am/insert.rs`
- `plan/tasks/06-graph-insert.md`
- `spec/functional/FR-016-hnsw-insert.md`
- `spec/adr/ADR-007-query-scoring-and-payload.md`

## What Landed

### 1. Insert-side invariants are now called out directly in code

`src/am/insert.rs` now spells out three previously implicit rules:

- the HNSW level sampler keeps the `+1` numerator so `bits = 0` cannot hit
  `ln(0)`
- insert-side beam ordering negates the inner-product scorer because
  `BeamCandidate` is "lower is better"
- metadata promotion only runs after append, because `entry_point` must always
  reference a live element at `metadata.max_level`

These are comment-only clarifications, but they address the exact "future
reader could simplify this incorrectly" concerns from the outside review.

### 2. The `ef_construction` fallback is now explicit instead of open-coded

The two insert-side `.max(1)` call sites now flow through a shared
`insert_ef_construction(...)` helper.

That helper:

- `debug_assert!`s that validated indexes should persist `ef_construction >= 1`
- still uses `.max(1)` as a corruption/legacy fallback in release builds

This keeps the runtime behavior unchanged while making the invariant visible.

### 3. The construction-vs-scan scoring split is now durable documentation

The A5 task doc, `FR-016`, and `ADR-007` now all say the same thing:

- bulk build and live insert use `score_code_inner_product`
- that construction metric is symmetric and MSE-only
- ordered scan continues to use the gamma-aware prepared raw-query scorer
  (`score_ip_from_parts`)

This moves the rule from "implicit across `build.rs` and `insert.rs`" to a
documented design choice.

### 4. The task doc now records two follow-up boundaries explicitly

`plan/tasks/06-graph-insert.md` now also records:

- pre-sized live-insert neighbor tuples leave unused slots as `INVALID`, and
  runtime traversal already has regression coverage for skipping those
  placeholders
- insert levels are deterministic per `(seed, heap_tid)` today, so future
  delete/rewrite work must revisit that if heap TIDs can be recycled

That gives future insert/vacuum/throughput work a concrete reminder instead of
relying on local reviewer memory.

## Validation

- `cargo test`
- `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

All passed on this checkpoint.

## Review Focus

- Is `insert_ef_construction(...)` the right narrow expression of the invariant,
  or should this branch be stricter than `debug_assert!` in v0.1?
- Do the `FR-016` / `ADR-007` updates capture the build+insert vs scan scoring
  split clearly enough, or is there still a missing design note?
- Is `plan/tasks/06-graph-insert.md` now explicit enough about the `INVALID`
  placeholder contract and the heap-TID-reuse caveat for future work?
