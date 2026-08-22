# Task 222: ec_distann Qual-Aware Payload Projection

Status: **proposed** (2026-08-21). Priority: P0 latency.

Program ledger: `plan/design/ec-distann-recall-latency-roadmap.md`, candidates
MAT-28 and MAT-29. Origin: Task 218's production lazy-10 attribution and the
Task 220/221 negative owner-materialization screens.

## Why

The production lazy-10 100k profile materializes only 6.66 remote rows per
scan, yet requests four columns per row, returns 123,076.8 payload bytes per
scan, and spends 8.752 ms/scan in owner payload SQL. The standard latency query
projects only `id`. `build_payload_metadata` currently ships every non-dropped
column because an older target-list-only implementation omitted qual columns
and evaluated remote predicates against NULL.

Whole-row shipping is correctness-safe but is now the strongest measured
avoidable materialization cost. The replacement must derive the complete set
of attributes the executor can read, not merely restore the rejected
target-list-only shortcut.

## Goal

Implement and measure a fail-closed, qual-aware payload attribute mask that
ships only columns required by the scan target list, scan/recheck quals, and
other executor-visible expressions. Ambiguous whole-row, system-column, or
unsupported expression shapes fall back to the current all-column behavior.

## Entry gate

1. Task 217's same-generation arm attestation remains mandatory.
2. The control is the conforming sharded owner path with lazy-10, schema cache,
   Algorithm-1 pushdown, and current gateway-copy behavior.
3. Pre-register the attribute-derivation contract and correctness matrix before
   measuring latency.

## Scope

### P1 — Attribute-use contract and observability

- Enumerate base-relation attributes referenced by target lists, executor quals,
  recheck expressions, and required junk/system-column paths.
- Preserve a conservative all-column fallback for whole-row Vars or any shape
  whose executor use cannot be proved.
- Emit requested attribute numbers, payload-column count, payload bytes, and
  fallback reason in benchmark-only evidence.
- Classify a true zero-payload/index-only path, but do not implement it unless
  every output and visibility value can be reconstructed without a row fetch.

### P2 — Correctness and isolated 100k A/B

Exercise id-only, multi-column, `SELECT *`, qual-only columns, nulls, toasted
values, mixed local/remote rows, correlated LATERAL queries, multi-window qual
rejection, rescan, and remote failure. Then run one same-generation 100k A/B in
which only the payload attribute mask differs.

### P3 — Full decision matrix

Advance only a useful end-to-end result to the standard 10k/50k/100k release
`ecaz bench suite` matrix. Report recall/result identity, mean and tails,
payload columns/bytes, owner endpoint/SQL time, storage, topology, and
NFR-021/NFR-022 conformance.

## Non-goals

- Repeating Task 220's packed SQL or Task 218's typed-locator candidate.
- Omitting qual/recheck attributes to win a benchmark.
- Changing traversal, head policy, payload window size, or stored format.
- Treating a candidate win as a shipped default without a separate
  productionization disposition.

## Acceptance

1. The attribute-use contract is test-pinned and fails closed to all-column
   shipping for ambiguous shapes.
2. All semantic/failure cases match the control, including qual behavior that
   broke the historical target-list-only attempt.
3. The 100k A/B proves whether column/byte reduction moves end-to-end latency.
4. A useful candidate receives 10k/50k/100k recall, latency, and storage
   evidence; otherwise the task closes STOP without a matrix.

## Required review packets

1. `reviews/task-222/001-plan/`
2. `reviews/task-222/002-contract-and-correctness/`
3. `reviews/task-222/003-isolated-100k/`
4. `reviews/task-222/004-full-scale-decision/` (only after a useful screen)

## References

- `reviews/task-218/001-production-profile-attribution/`
- `reviews/task-220/002-isolated-candidate/`
- `src/am/ec_distann/custom_scan.rs` (`build_payload_metadata`)
- Roadmap MAT-28 / MAT-29
