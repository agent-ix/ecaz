# Task 237: ec_distann Protocol Error and EXPLAIN Observability

Status: **ready — Tasks 234 and 236 prerequisites complete; run before Task
228** (updated 2026-08-26). Task 234's operator ACCEPT is recorded in
`reviews/task-234/005-50k-reuse-drift-bound/feedback/2026-08-26-03-operator.md`.
Priority: P1 correctness/operations.

## Why

FR-081 requires per-query rounds, records expanded, code-scored candidates,
per-node batch sizes, and pool reuse in EXPLAIN. The custom scan currently has
no `ExplainCustomScan` implementation; some counters exist only behind a debug
NOTICE or benchmark-only feature.

The protocol also has inconsistent fail-closed behavior. The physical
generation reader raises `EC_GENERATION_MISSING` for a missing row-tier
payload, while another custom-scan path can silently drop the missing remote
row. Distributed errors are assembled through several string paths, some of
which include raw remote details. Operators therefore cannot reliably tell
epoch retry from placement fault, structural corruption, timeout, cancellation,
authentication, or backend termination without enabling non-production
instrumentation.

## Goal

Give every production ec_distann distributed scan a stable sanitized error
taxonomy and always-available bounded EXPLAIN counters for head, traversal,
materialization, transport, pool, retry, and failure behavior. Make missing
owned graph/vector/row-tier data fail closed exactly as FR-079 and NFR-020
specify.

## Entry conditions

1. Task 234 supplies typed timeout/cancel outcomes for all read RPC classes.
2. Task 236 supplies sanitized secure-connection failure categories before the
   final taxonomy is frozen.
3. The plan packet maps every existing `EC_*`, SQLSTATE, transport error,
   retry/restart path, debug counter, benchmark feature counter, and
   `ExplainCustomScan` gap.

## Required implementation

### P1 — Stable fail-closed error contract

- Define typed internal variants and stable SQL-visible categories for bad
  input/version, endpoint/owner/placement mismatch, epoch mismatch/retry,
  missing graph record, missing exact-vector row, missing final row-tier
  payload, corruption/schema drift, connect/auth/TLS, remote statement timeout,
  local timeout/cancel, backend termination, and generic sanitized transport.
- Preserve the retriable/non-retriable distinction. Structural absence inside
  a Published retained generation is non-retriable corruption/co-placement
  drift, not an ordinary concurrent-delete filter.
- Remove silent dropping of a missing owned payload. Tombstones and legitimate
  scan exhaustion remain distinct, specified non-error outcomes.
- Do not expose raw conninfo, secrets, row/source bytes, or unsanitized remote
  server errors.

### P2 — Production EXPLAIN surface

- Implement `ExplainCustomScan` for ec_distann and emit bounded per-query
  counters in text and JSON EXPLAIN: head owners/requests/seeds, rounds,
  records expanded, candidates scored/returned, per-owner batch summary,
  gateway skips, materialization windows/rows/bytes, pool opens/reuse/evictions,
  retries, timeouts, cancels, and stable failure category.
- Keep labels/versioning stable and make counters available in normal release
  builds without debug NOTICE or benchmark feature flags.
- Summaries must be bounded independently of owner count/query history where
  NFR-021 requires it; do not emit raw vec_ids, payloads, conninfo, or an
  unbounded per-RPC trace.
- Reconcile EXPLAIN totals with Task-228 suite/structured metrics so the two
  surfaces cannot silently disagree.

### P3 — Differential and fault evidence

- Add local/remote and text/JSON EXPLAIN fixtures with exact label/counter
  assertions for early exit, full H rounds, gateway copy, lazy-10 deepening,
  pool reuse, and epoch restart.
- Inject missing graph, vector, and final row-tier payloads and prove each maps
  to its distinct structural fault without returning a shortened result.
- Cover timeout/cancel/TLS/auth/backend faults after Tasks 234 and 236 and
  inspect every output surface for sanitization.
- Add suite parsing/assertions for the production EXPLAIN fields needed by
  Task 228; extend `ecaz bench suite`, not a packet-local parser script.

## Non-goals

- High-cardinality distributed tracing or logging every candidate/RPC.
- Changing search budgets, materialization windows, retry policy, or degraded
  completion semantics.
- A new wire format or transport implementation.
- Treating EXPLAIN timing as a substitute for suite-produced benchmark
  evidence.

## Acceptance

1. Missing owned graph/vector/payload cases fail with distinct stable
   non-retriable categories and return no partial rows.
2. Normal release text and JSON EXPLAIN expose the FR-081/NFR-019 counters with
   bounded cardinality and no secrets/payloads.
3. EXPLAIN, suite metrics, and fault outcomes reconcile on the same PG18 runs.
4. Outside review accepts the error taxonomy, sanitization audit, and
   FR-081-AC-5 observability claim.

## Required review packets

1. `reviews/task-237/001-plan-taxonomy-and-counter-map/`
2. `reviews/task-237/002-error-conformance/`
3. `reviews/task-237/003-explain-and-suite-surface/`
4. `reviews/task-237/004-pg18-fault-closeout/`

## References

- FR-079 missing-record/vector/payload behavior
- FR-081 implementation gap F8 and FR-081-AC-5/6
- NFR-014, NFR-019, NFR-020, NFR-021
- Tasks 214, 228, 234, 235, and 236
- `src/am/ec_distann/custom_scan.rs`
- `src/am/ec_distann/generation_read.rs`
- `src/am/ec_distann/remote_transport.rs`
