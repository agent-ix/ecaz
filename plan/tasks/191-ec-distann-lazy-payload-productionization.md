# Task 191: ec_distann Lazy Payload Productionization

Status: **proposed, unblocked by Task 184 PROMOTE** (2026-07-19). Priority: P1.

Decision source: ADR-085 D12. Evidence source:
`reviews/task-184/003-isolated-candidate/` and
`reviews/task-184/004-full-scale-decision/`.

## Why

Task 184's fixed batch-10 candidate preserved distinct recall and required
semantics while reducing physical warm mean latency by 38.3% to 41.5% at
10k/50k/100k and remote payload bytes by 72% to 75%. The implementation is
currently compiled only with `distann-head-attribution-benchmark`, controlled
by `ec_distann.benchmark_materialization_batch_size`, and normal builds retain
the eager path. The measured winner therefore is not yet a production result.

## Goal

Make ADR-085 D12's executor-driven, deterministic global-ranked payload windows
of exactly 10 the normal physical `ec_distann` scan path. Preserve result
identity, ordering, projection and qual semantics, snapshot/generation fencing,
remote failure behavior, and bounded work. Change no persisted index/row format,
placement rule, payload endpoint, codec, head policy, or traversal budget.

## Contract checkpoint

Before the default change:

1. amend FR-079/FR-081, NFR-019, and the relevant test matrix to state when
   payload work is triggered, how deterministic window deepening interacts
   with quals, and that a later owner error aborts the query rather than
   returning a prefix; replace NFR-019's unconditional `payload reads <= k`
   wording with the exact unqualified and qual-driven bounds selected below;
2. pin the production batch size to 10 as an internal policy, not a reloption or
   user-tunable production GUC;
3. retain eager materialization only as a benchmark/test A/B override that is
   absent from normal production builds; and
4. state the rollback contract: installing the prior extension restores eager
   scans without an index rebuild because D12 changes no durable bytes.

Task 184's outside review adds four required carryovers before the production
gate:

5. make the wide-varlena scenario prove genuine out-of-line TOAST storage, not
   merely compressed-inline detoast; use incompressible content and/or
   `STORAGE EXTERNAL`, and assert the fixture's storage shape before comparing
   eager and lazy outputs;
6. preserve materialized-but-unconsumed remote payloads in the stable prefix
   across search deepening, and assert that one `vec_id` is not remotely fetched
   twice solely because a rebuild reset its slot to pending;
7. remove the `output_merge` / `materialize_output_associate` double-booking or
   emit explicit machine-readable alias metadata so reporters cannot add both
   as independent work; and
8. capture the suite runner Git descriptor before creating or updating tracked
   artifact outputs, eliminating the runner's self-inflicted `-dirty` state.

## Implementation

- Move the proven lazy slot/deepening implementation onto the unconditional
  physical scan path and make fixed 10 the normal behavior.
- Keep one code path for production lazy10 and the benchmark lazy10 arm; do not
  fork or reimplement the algorithm for release builds.
- Make the feature-gated benchmark override able to select the eager control
  without changing the normal-build default.
- Preserve the same projection attnums, concurrent owner endpoint, epoch/schema
  validation, memory-context datum ownership, stable `vec_id` identity, global
  ordering, and bounded ranked candidate set.
- Preserve the current corpus-independent query cap: each batch ends at the
  current proven prefix, each batch contains at most 10 global-ranked slots,
  and total qual-driven payload reads cannot exceed the deepening ceiling fixed
  once from the initial search bar (`max(initial × 64, 1024)`). Add counter
  assertions for this bound; do not introduce a new work-cap choice.
- Carry already-materialized payloads forward when a deepened search rebuilds
  a stable ranked prefix. Preserve rank, identity, datum ownership, and failure
  semantics; this is a redundant-work fix, not a cache with cross-query or
  cross-generation lifetime.
- Keep Task 184's attribution/work counters feature-gated unless a narrowly
  justified production diagnostic is explicitly reviewed.

## Correctness and failure evidence

Run focused PG18 unit/pgrx coverage and the `ecaz bench suite` semantic matrix
for:

- eager/production output identity without quals;
- filters rejecting the first window and multiple windows;
- null and genuinely out-of-line toasted/varlena projected and qualified
  columns, with fixture-shape evidence;
- mixed local/remote winners and tie ordering;
- fewer than, exactly, and more than 10 ranked candidates; and
- an owner outage after the first cursor batch, which must fail closed.

The multiple-window qual case must also prove that stable-prefix rebuilds do
not re-request a previously materialized remote `vec_id`.

Also prove a normal production build has no benchmark GUC while using
the same lazy10 driver, and that the feature build's eager override is not a
production default surface.

## Benchmark gate

Run a checked-in `ecaz bench suite` matched eager-control versus production
lazy10 A/B at 10k/50k/100k on one byte-identical generation per scale. Minimum
shape is 200 held-out queries / 2,000 distinct top-10 trials and 50 warm latency
samples after 10 warmups at concurrency one. Record recall/Wilson interval,
mean/p50/p95/p99/max, materialization stage/work/bytes, storage, construction,
topology, remote engagement, query separation, and unanimous installed release
provenance.

The final suite manifest must carry a clean runner descriptor without relying
on a prose exception, and stage results must either be non-overlapping or
machine-readably identify aliases.

Promote only if the production path reproduces Task 184's recall/semantic
parity and material end-to-end mean and tail improvement at every required
scale. Use relative Pareto evidence; do not invent an absolute latency gate.

## Required review packets

1. `reviews/task-191/001-production-contract/`: ADR/FR/test contract and exact
   default/override/rollback choices;
2. `reviews/task-191/002-production-implementation/`: code checkpoint plus
   focused PG18 and semantic/failure evidence;
3. `reviews/task-191/003-production-full-scale/`: checked-in 10k/50k/100k
   eager-vs-production suite and promote/iterate/stop decision; and
4. `reviews/task-191/004-closeout/`: retained baseline, Task 187 handoff,
   outside feedback dispositions, and final requirement audit.

## Non-goals

- Adaptive, 20/40, pipelined, speculative, or projection-pruning variants.
- Traversal transport work owned by Task 187.
- Head, graph, codec, placement, or protocol changes.
- A production tuning knob for the batch size.
- Treating Task 184's benchmark-feature build as sufficient release evidence.
- Cross-query payload caching; the stable-prefix reuse above is scan-local.

## References

- ADR-085 D12; FR-079 and FR-081.
- Task 184 packets 002--004.
- NFR-007 and NFR-017 through NFR-020.
- `plan/design/ec-distann-recall-latency-roadmap.md`.
