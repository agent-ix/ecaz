# Task 233: ec_distann Hybrid Node/Columnar Generation

Status: **proposed — operator-selected mandatory integration prototype after
Task 232** (2026-08-22). Priority: P2 storage/retrieval architecture.

Program ledger: `plan/design/ec-distann-recall-latency-roadmap.md`, candidate
ARCH-18. Tasks 229--232 establish isolated mechanism effects. This task tests
the interaction that those attribution arms deliberately prohibit.

## Why

Graph traversal and SQL result projection consume different physical units.
Traversal sparsely reads nearly a whole graph node: exact vector, adjacency,
search code, and neighbor codes. Result materialization reads a few winning
ordinals and often only one or two payload attributes. Task 231 therefore
tests fixed-stride whole-node extents, while Task 232 tests per-attnum packed
segments.

The likely hybrid is not the sum of their storage footprints. The exact vector
belongs authoritatively in the Task-231 node extent and must be omitted from the
columnar payload surface. Both surfaces share one owner-local dense ordinal;
Task 222's attribute mask routes vector projection to the node extent and other
requested attributes to their column segments. Only a factorial experiment can
show whether the traversal and projection gains survive together or interfere
through cache residency, I/O amplification, construction, or DML work.

## Goal

Implement and benchmark an opt-in generation format combining fixed-stride
graph/vector extents with packed non-vector payload columns and a transactional
delta overlay. Compare all four graph/payload combinations at 10k, 50k, and
100k so the interaction has direct evidence and select a reviewed final storage
disposition without duplicating the exact vector.

## Entry conditions

1. Tasks 222--224 and 229--232 are review-closed with packet-local evidence.
2. The Task-231 and Task-232 opt-in implementations remain available even when
   either isolated decision is STOP. Constituent STOPs do not skip this task;
   an interaction can differ from either main effect.
3. A design packet freezes the common ordinal/directory identity, vector
   authority, page/segment versions, overlay routing, and factorial suite arms
   before implementation.

## Required implementation

### P1 — One generation, two read-optimized base surfaces

- Use one owner-local dense ordinal for the fixed-stride node extent and every
  packed payload segment. A directory lookup resolves vec_id and base/overlay
  state once; it must not introduce a second graph-heap or per-column B-tree
  probe.
- Store graph header, full-precision exact vector, search code, neighbor ids,
  and neighbor codes in the fixed-stride extent defined by Task 231.
- Store every non-vector source attribute in exactly one Task-232 column
  segment. Do not persist a second exact-vector column or a duplicate frozen
  row for fallback convenience.
- Route exact-vector SQL projection to the node extent and non-vector
  projection/quals to only the segments required by Task 222's fail-closed
  mask. Whole-row reconstruction restores original attnum order exactly.
- Keep the public source table on PostgreSQL's normal heap. The hybrid is
  internal generation storage on PostgreSQL-managed relation pages with the
  repository's established buffer, page-lock, digest, and WAL discipline; do
  not require a public Table AM, external files, or mmap.

### P2 — Lifecycle and mutable overlay

- Bind the node relation, segment descriptors, common ordinal map, schema/type
  identities, and all digests into one receipt, manifest, epoch fingerprint,
  publication, retention, retirement, and reclaim unit.
- Base generations are immutable. Task 167 inserts and replacements append a
  new fixed-stride node/vector extent and place only non-vector payload values
  in the transactional delta heap; atomic directory publication selects the
  new node plus base-columnar or delta-payload location. Deletes retain the
  existing tombstone and generation-fencing contract.
- Reads spanning base columns, delta payloads, and appended node extents are
  snapshot-safe and byte-identical. The next epoch rebuild compacts payload
  overlays and node appends without ever exposing a partial surface set.
- Old row-heap/current-graph generations remain readable and rollback requires
  no data migration.

### P3 — Factorial evidence and synthesis

- Add one checked-in `ecaz bench suite` configuration with the same corpus,
  query hashes, search policy, generation inputs, release provenance, and
  instrumentation for all four arms: current graph + row payload, fixed-stride
  graph/vector + row payload, current graph + columnar row tier, and
  fixed-stride graph/vector + non-vector columnar payload.
- Run every arm at 10k, 50k, and 100k. Include standard warm and
  controlled-residency profiles plus traversal/rerank-only, id-only, narrow
  scalar, exact-vector projection, mixed, cold/wide, and `SELECT *` workloads.
- Report main effects and the interaction separately: graph/vector and payload
  reads/bytes/hits, directory probes, owner stages, decode/reconstruction CPU,
  wire bytes, recall/result identity, p50/p95/p99/max and throughput, build and
  handoff time, DML/overlay/compaction cost, padding, per-surface and total
  storage, and NFR-021/NFR-022 conformance.
- Reconcile the factorial result with Tasks 229 and 230 in a final comparison
  table. Do not select a default from predicted complementarity or from a
  single workload/scale.

## Decision rule

The implementation and full factorial 10k/50k/100k matrix are mandatory even
if Task 231 or Task 232 individually closes STOP. PROMOTE the hybrid only when
it preserves exact results/recall, demonstrates a material end-to-end benefit
over both single-mechanism arms on a declared production workload, has no
unexplained workload regression, and its total storage/build/DML/operations
cost is acceptable. Otherwise close STOP and use the cross-task synthesis to
retain the current layout or advance the strongest isolated candidate. Any
default flip receives a separate productionization task and release A/B.

## Non-goals

- A columnar graph or adjacency representation.
- A public PostgreSQL Table AM or replacement of the user's heap table.
- Graph/community reordering, prefetch, changed BW/H/L, codec changes, or a
  coordinator-resident O(N) copy.
- Keeping both columnar and fixed-stride copies of the exact vector.
- Treating Task-231 plus Task-232 historical numbers as a substitute for the
  same-run factorial matrix.

## Acceptance

1. One ordinal and one authoritative copy per field are format- and
   corruption-test-pinned, including vector projection and whole-row rebuild.
2. Atomic publication, restart, rollback, DML overlay, retirement, and reclaim
   cover both surfaces as one generation on PG18.
3. The four-arm suite reports 10k/50k/100k recall, latency, storage, build, and
   DML evidence with main-effect and interaction attribution.
4. Outside review accepts a PROMOTE or STOP hybrid verdict and a comparative
   disposition covering Tasks 229--233.

## Required review packets

1. `reviews/task-233/001-plan/`
2. `reviews/task-233/002-composed-format-and-reader/`
3. `reviews/task-233/003-lifecycle-overlay-and-correctness/`
4. `reviews/task-233/004-factorial-decision/`

## References

- Tasks 167, 179, 204, 222--224, and 229--232
- Roadmap ARCH-05, ARCH-17, and ARCH-18
- ADR-045 and ADR-085 D1/D11
- `DISTRIBUTEDANN` sections 2.1--2.3 (arXiv:2509.06046)
- PostgreSQL table-AM, TOAST, page-layout, and extension-WAL contracts
- FR-076, FR-078, FR-079, FR-082, FR-083
- NFR-007, NFR-016, NFR-018, NFR-021, NFR-022


