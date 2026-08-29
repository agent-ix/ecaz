# Task 232: ec_distann Packed Columnar Immutable Row Tier

> **Tracking moved to GitHub (2026-08-29):** [agent-ix/ecaz#98](https://github.com/agent-ix/ecaz/issues/98)
> on [Project 19](https://github.com/orgs/agent-ix/projects/19), under EPIC #95.
> The Status header below is frozen; status updates land on the issue.
> Review packets remain under `reviews/task-232/`.

Status: **proposed — operator-selected mandatory prototype, last in the layout
sequence** (2026-08-22). Priority: P2 storage/retrieval architecture.

Program ledger: `plan/design/ec-distann-recall-latency-roadmap.md`, candidate
ARCH-05. This is deliberately last because it replaces the general frozen-row
representation and has the broadest format, DML, and recovery surface.

## Why

Task 222 makes the required attribute set explicit. A row heap still stores and
retrieves data as tuples, while the DistANN result path often needs only one or
two attributes from a handful of ranked rows. A packed immutable columnar tier
can read only those attributes and can persist canonical binary values once at
build/handoff rather than invoking PostgreSQL send functions for every query.

Columnar storage is not automatically better for graph traversal: expansion
needs the exact vector and full adjacency as a whole-node unit, which is why
Task 231 tests fixed-stride nodes first. This task isolates columnar benefits to
the frozen row/payload tier and uses the Task-231 graph winner only as a
separately reported secondary comparison, never in the primary attribution arm.

## Goal

Implement and benchmark an opt-in, generation-owned columnar row tier keyed by
owner-local dense ordinal, with per-attnum null maps and fixed- or
variable-width segments. Prove whether selective reads and build-time binary
encoding beat the current PostgreSQL row heap across narrow and wide query
shapes.

## Entry conditions

1. Tasks 222--224 and 229--231 are review-closed and their evidence is available.
2. The primary control remains the frozen post-224 row-heap layout; prior layout
   winners are not stacked into the primary A/B.
3. The design packet freezes segment/page format, ordinal mapping, chunking,
   checksum/digest coverage, schema evolution, and DML overlay behavior.

## Required implementation

### P1 — Column segment format

- Persist a generation descriptor mapping original attnums to typed column
  segments and their canonical PostgreSQL binary I/O identity.
- Fixed-width columns use dense value arrays plus a null bitmap. Variable-width
  columns use a null bitmap, bounded offset table, and chunked value segments;
  offsets and lengths are endian-explicit and independently validated.
- Persist the exact vector as a dedicated fixed-shape segment suitable for
  bounded batch reads. Other attributes are read only when Task 222's mask
  requests them.
- Use PostgreSQL-managed relation pages/WAL. Do not depend on process-local
  files, raw mmap, or PostgreSQL TOAST pointers as the durable column format.
- Store canonical typsend bytes at handoff/build time and reconstruct through
  the validated typreceive contract. Unknown or changed type I/O identity
  fails closed.

### P2 — Read, lifecycle, and overlay

- Resolve vec_id to dense ordinal, gather requested ordinals by column, and
  restore exact global rank/attnum order after batched reads.
- Bind every segment descriptor and digest into receipt, manifest, epoch
  fingerprint, topology, storage, publication, and reclaim machinery.
- Use a transactional row-heap delta overlay for Task 167 inserts and
  replacements, with the directory recording base-columnar versus overlay
  location. Deletes retain the existing tombstone semantics. The next epoch
  rebuild compacts the overlay into base segments.
- Reads spanning base and overlay must be byte-identical, snapshot-safe, and
  generation-fenced. Recovery cannot expose only a subset of column segments.
- Old row-heap generations remain readable and rollback requires no data
  migration.

### P3 — Evidence

- Cover fixed-width, variable-width, NULL, empty, external-TOAST source,
  dropped-column, domain/collation, wide vector, mixed narrow/wide projection,
  qual-only, `SELECT *`, deepening, rescan, base/overlay mixture, restart,
  partial-build failure, retirement, and corruption cases.
- Run isolated row-heap versus columnar A/B at 10k, 50k, and 100k with a
  checked-in `ecaz bench suite` config. Include id-only, narrow scalar,
  vector-bearing, mixed, and whole-row workload profiles.
- Compare arms at matched fixture position, or use a preregistered
  counterbalanced envelope that separates position/warmth from the candidate.
  Never compare a fresh-build control only against a reused candidate.
- Report segment reads/bytes by attnum, null/offset/value work, exact-vector
  reads, owner CPU/wall, wire bytes, end-to-end latency/tails, recall/result
  identity, build/handoff time, DML overlay cost, compaction estimate, per-tier
  storage, and NFR conformance. Attribute exact-vector-segment work separately
  from non-vector payload-column work so Task 233 can test their composition
  without treating an aggregate Task-232 verdict as the interaction result.
- Report a secondary non-decision comparison against any Task-229--231 winner,
  but do not use a stacked arm to claim columnar attribution.

## Decision rule

The prototype and full 10k/50k/100k matrix are mandatory. Promotion requires a
material end-to-end benefit on at least one declared workload class, no
unexplained regression on the others, bounded overlay growth, and acceptable
storage/build/operations cost. Otherwise close STOP and retain the simpler
layout selected by earlier tasks. Continue to Task 233 regardless and retain
the opt-in prototype until its mandatory factorial hybrid experiment closes.

## Non-goals

- Replacing graph adjacency with a columnar graph representation.
- Combining fixed-stride graph blocks with the primary columnar A/B; Task 233
  owns that separate composition and removes the duplicate exact-vector
  segment in its hybrid arm.
- An external columnar extension dependency or non-WAL sidecar.
- Claiming benefit from byte reduction without end-to-end movement.

## Acceptance

1. All segment formats have golden fixtures, independent decoders, overflow,
   truncation, checksum, endian, and unknown-version tests.
2. Narrow and whole-row SQL semantics are byte-identical across base and DML
   overlay rows.
3. Publication/recovery/retirement treats the segment set atomically.
4. Full-scale recall, latency, storage, build, and DML evidence supports a
   reviewed PROMOTE or STOP disposition and comparison with Tasks 229--231.

## Required review packets

1. `reviews/task-232/001-plan/`
2. `reviews/task-232/002-hybrid-handoff/`
3. `reviews/task-232/003-format-and-reader/`
4. `reviews/task-232/004-overlay-lifecycle-and-correctness/`
5. `reviews/task-232/005-full-scale-decision/`

## References

- Tasks 222--224 and 229--231; Task 233 consumes the isolated evidence
- Roadmap ARCH-05
- FR-076, FR-078, FR-079, FR-082, FR-083
- NFR-007, NFR-016, NFR-018, NFR-021, NFR-022
- PostgreSQL TOAST and binary type I/O contracts

