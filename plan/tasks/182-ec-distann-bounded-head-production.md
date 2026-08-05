# Task 182: ec_distann Bounded-Head Production Implementation

> **MULTI-NODE MEASUREMENT RULE (NON-NEGOTIABLE).** Any decision about
> distributed behavior — latency, recall, storage, or overhead — MUST be measured
> on a multi-node configuration. A single-node / single-instance arm is NEVER
> acceptable as the basis for a decision about a distributed algorithm; its only
> permitted use is a clearly labeled baseline that quantifies distribution
> overhead. Label every reported number with its arm's node count. See
> AGENTS.md → "Distributed Measurement: Multi-Node Arms Only".

Status: **completed — outside-reviewed PROMOTE explicit trained policy**
(2026-07-16; review ACCEPT 2026-07-17). The
bounded 4,096 training-landmark, exact-scoring policy is implemented and
attested on the normal production build/read path. Production A/B reproduced
Task 181's recall: neutral at 10k, +0.0140 at 50k, and +0.0350 at 100k, with
acceptable matched latency and effectively unchanged storage/build cost.
Legacy/current builds remain byte-compatible and the default because trained
builds require an explicit disjoint training relation. The owner oracle remains
diagnostic. Outside acceptance of code, format/ADR, tests, and benchmark
evidence is recorded in
`reviews/task-182/007-closeout/feedback/2026-07-17-01-reviewer.md`. Task 183
owns residual recall and latency work. Depends on Tasks 179, 180, and 181.

## Why

Task 180 proved that wider search, more seeds, and linear head-cap growth do not
recover the bounded-head quality gap. Task 181 is reserved for coverage-aware
landmark and bounded-hierarchy measurement without production changes. This
task provides the separate implementation boundary required by Task 180: a
winner is not production-ready merely because a benchmark-only arm won.

Do not start this task from a promising intermediate Task 181 cell. Its input
must be Task 181's final reviewed packet, including the exact deterministic
policy, all query-work caps, digests, storage/cache/build accounting, and
full-scale gate evidence.

## Goal

Implement Task 181's one approved bounded-head candidate in the production
physical-generation build and read paths, preserve bounded query work and
lifecycle correctness, and make a separately reviewed promote/iterate/abandon
decision from production-path A/B evidence.

If Task 181 closes NO-GO on relative A/B evidence, mark Task 182 `won't
pursue` without implementation.

## Entry gate

Before any code change, the corrected entry packet (packet 003) and the first
implementation packet must cite and verify all of the following:

1. Task 181's final decision identifies a GO candidate;
2. one candidate is named with no unresolved algorithm or parameter choice;
3. the candidate demonstrates reproducible recall improvement over unchanged
   production where bounded-head coverage is deficient without regressing the
   remaining measured scale;
4. matched 10k/50k/100k warm latency and storage are reported, including any
   regression as well as improvement;
5. all per-query work and per-level memory/storage caps are explicit;
6. no query-time owner scan, uncapped fanout, or evaluation-query training is
   involved; and
7. its benchmark-only implementation has deterministic build/output digests.

Failure of any entry item blocks implementation and returns the decision to a
new measurement task; do not redesign the candidate inside Task 182. Proposed
NFR-017 targets remain comparison points and do not independently block entry.

## Production design checkpoint

Translate the frozen candidate into a durable production contract before
wiring it into scans:

- deterministic landmark/hierarchy construction inputs and tie-breaks;
- on-disk metadata and versioning, including explicit old-index behavior;
- generation descriptor/fingerprint coverage for all policy parameters and
  artifacts;
- bounded builder memory and spill behavior;
- build, Ready, publish, retire, recovery, and scan-pin ownership of every new
  relation/artifact;
- query-time level/region/landmark/seed caps;
- physical-owner routing and remote failure semantics;
- storage/cache accounting and inspection output; and
- upgrade, rollback, and rebuild requirements.

Add or amend an ADR when the winner changes persisted layout, generation
fingerprints, upgrade semantics, or default policy. A clean format break is
acceptable only when stated and tested; silent reinterpretation is not.

## Implementation requirements

1. Production builds must not require
   `distann-head-attribution-benchmark` or expose benchmark GUCs/endpoints.
2. The selected policy must be encoded in reloptions/metadata where needed and
   attested by inspect/benchmark output; defaults change only at the final gate.
3. Query work must fail closed if metadata would exceed a declared cap. No
   fallback may silently perform an owner-wide scan or unbounded remote fetch.
4. Physical generation publication is atomic across the new head artifacts and
   existing graph/row/directory artifacts. Partial Ready/Published state is
   never readable.
5. Retirement/reclaim waits for registered readers and removes all policy
   artifacts without residue.
6. Remote expansion/materialization behavior, BW/H/top-k, RaBitQ neighbor
   scoring, and source-identity semantics remain unchanged unless Task 181's
   reviewed winner explicitly included a triggered residual-traversal change.
7. Existing indexes either retain their old production behavior or fail with a
   clear rebuild requirement according to the reviewed format contract.

Benchmark-only owner-scan and exact-neighbor surfaces remain diagnostic and
must not become a production fallback.

## Correctness validation

Use the primary PG18 lane and add the narrowest coverage that proves:

- deterministic builder/artifact digests across repeated builds;
- scalar/reference equivalence for landmark selection and routing;
- exact enforcement of every query-work cap;
- no evaluation-query data embedded in production artifacts;
- local and three-owner result identity/recall invariants;
- restart/recovery at pre-Ready, Ready, publish-decision, post-publish, retire,
  and reclaim boundaries;
- missing/corrupt/mismatched policy artifacts fail closed;
- old-index compatibility or explicit rebuild errors;
- concurrent readers pin the correct generation; and
- no orphan relations, rows, records, or policy metadata after rollback and
  reclaim.

Run focused unit tests and focused `cargo pgrx test pg18` coverage where
PostgreSQL callback/lifecycle behavior is involved. PG17 remains optional unless
the selected format or callbacks are PG17-facing.

## Production-path benchmark gate

Drive all A/B measurement through checked-in `ecaz bench suite` configs. On one
fresh generation per scale, compare:

1. unchanged Task 180 production baseline;
2. the production implementation of Task 181's candidate; and
3. benchmark-only owner oracle as a diagnostic reference.

Run 10k/50k/100k minimum with 200 held-out queries / 2,000 distinct top-10
trials and 50 warm latency measurements after 10 warmups at concurrency 1.
Record recall/CI, p50/p95/p99/max, build/publish time, all physical/control/
source/single-index bytes, every head level's bytes/cache, topology, remote
engagement, and unanimous installed release provenance.

The candidate must reproduce Task 181's quality and latency on the normal
production path. A benchmark-feature result cannot substitute for this gate.

## Promotion decision

Promote the selected production policy/default only if it:

1. preserves all correctness, lifecycle, recovery, and bounded-work invariants;
2. reproduces Task 181's recall improvement over unchanged production without
   a recall regression at another measured scale;
3. retains an acceptable matched-latency and storage tradeoff, with the
   proposed `0.9990` recall and `37.6 ms` IVF anchor reported separately as
   aspirational context rather than hard pass/fail criteria;
4. passes topology/provenance/remote-engagement gates at every scale;
5. has accepted build-time, total-storage, and cached-head costs reported rather
   than hidden;
6. reproduces Task 181 within overlapping recall intervals without a material
   latency regression; and
7. receives outside review of code, format/ADR, tests, and benchmark evidence.

If correctness holds but the relative performance improvement does not
reproduce, keep the policy default-off only when it has a concrete reviewed
experimental use; otherwise remove it and close `abandon`. Do not promote
owner-scan work.

## Required review packets

1. `reviews/task-182/001-entry-and-production-design/`: historical conditional
   design; its hard entry gate is superseded;
2. `reviews/task-182/002-wont-pursue-closeout/`: historical disposition,
   superseded;
3. `reviews/task-182/003-reopen-correction/`: corrected Task 181 GO verification
   and frozen candidate;
4. `reviews/task-182/004-builder-and-format/`: deterministic builder, production
   contract, metadata, storage, inspection, compatibility, and any format/ADR;
5. `reviews/task-182/005-query-and-lifecycle/`: production read path, caps,
   publication/recovery/retirement, and fault evidence;
6. `reviews/task-182/006-production-ab/`: 10k/50k/100k production-path A/B; and
7. `reviews/task-182/007-closeout/`: outside-reviewed promote/iterate/abandon
   decision and task/index status sync.

Every evidence packet follows NFR-007 provenance and repository artifact rules.
No corpus TSVs, truth caches, node logs, polling exhaust, or run directories are
committed.

## Non-goals

- Choosing or tuning a candidate that Task 181 did not freeze.
- O(N) owner scans, uncapped remote seeding, or evaluation-query training.
- General graph/codec replacement, OPQ research, or unrelated quantizer work.
- Task 167 incremental DML or Task 172 throughput/capacity/RTT programs.

## References

- Task 181 final decision packet: `reviews/task-181/006-decision-correction/`.
- Task 182 production A/B: `reviews/task-182/006-production-ab/`.
- Task 182 closeout: `reviews/task-182/007-closeout/`.
- Task 180 packets 002/003: bounded-head attribution and NO-GO.
- Task 179 physical-generation lifecycle closeout.
- FR-078 through FR-081: placement, physical reads, head, and orchestration.
- NFR-017: distinct-recall and matched-latency release gate.
- NFR-018 through NFR-020: storage, bounded work, and failure semantics.
- NFR-007: benchmark provenance.
