# Task 116: IVF Rerank Insert Group Batching

Status: **proposed**.
Priority: P1 format-independent rerank storage follow-up after Task 111h.
Source: Task 111h closeout residual reviewer note
`reviews/task-111h/048-final-closeout-decision-v9/feedback/2026-06-20-01-reviewer.md`.

## Goal

Fix the IVF index-side rerank insert path so it does not mint one-wide rerank
groups under normal insert workloads.

The durable model should be:

- a logical dense posting group has scorer width, not page capacity,
- physical segment tuples are fragments of that logical group,
- metadata is recorded once for the group, preferably in a header segment,
- continuation segments carry payload bytes only, or nearly only payload bytes,
- flush happens only at logical group completion, list boundary, live-TID budget
  exhaustion, or final tail,
- final partial groups are padded to scorer width while exposing only
  valid/live/nondeleted postings to scan,
- all rerank formats share the same storage/grouping architecture.

## Why

Task 111h fixed the old query-time sidecar rebuild problems and proved that
direct posting-to-rerank group lookup is viable. The remaining insert-side
problem is independent of the chosen rerank format: if every insert creates a
single-entry rerank group, the index accumulates many tiny groups with repeated
headers and poor scan locality.

That path can bloat index storage, increase cold I/O, and undo the intended
batched scoring shape even when the format itself is otherwise good. It also
makes future compact rerank formats harder to evaluate honestly because the
storage layout, not the codec, can dominate the result.

## Scope

- IVF `rerank_placement = 'index'` sidecar groups.
- Insert-time grouping for all supported and future rerank formats.
- Logical group metadata layout and continuation segment policy.
- Scan lookup compatibility with posting-carried `rerank_tid` or equivalent
  direct group references.
- Delete/update/vacuum visibility for partially live groups.
- Instrumentation that reports group width and insert-created group shape.
- Focused PG18 correctness coverage plus benchmark evidence through
  `ecaz bench suite`.

## Non-Goals

- Do not add a one-off f16-specific path.
- Do not change the coarse stage or nprobe policy.
- Do not promote any compact rerank format by argument; promotion still requires
  recall/latency/storage evidence.
- Do not fork the benchmark workflow into task-local shell sweepers.
- Do not implement a background repack worker unless a later task explicitly
  scopes it.

## Phases

### Phase 1 - Layout Audit

- Trace build, insert, update, delete, vacuum, and scan for index-side rerank
  groups.
- Confirm exactly where one-wide groups are minted today.
- Document current metadata repetition, group width distribution, and direct
  lookup invariants.
- Identify whether any existing mixed build-plus-insert index can be read after
  the change, or whether the format version must advance.

### Phase 2 - Common Logical Group Builder

- Add a common group builder used by build and insert paths where possible.
- Make the target logical group width the active scorer width.
- Flush only at logical group completion, list boundary, live-TID budget
  exhaustion, or final tail.
- Pad final partial groups internally, but expose only valid/live/nondeleted
  postings to scoring and heap output.
- Keep group metadata once per logical group: list id, live count or live mask,
  delete bitmap if needed, gamma/codec metadata, heap TID counts/offsets,
  rerank TIDs, and heap TIDs where the design requires them.

### Phase 3 - Segment Continuation Format

- Split logical group metadata from physical payload fragments.
- Make continuation segments payload-only, or document every non-payload byte
  that must remain there.
- Preserve direct posting-to-group lookup without rebuilding a query-time map.
- Add format-version and upgrade/rebuild handling consistent with NFR-016 if
  persisted bytes or field sizes change.

### Phase 4 - Mutation Semantics

- Cover insert, update, delete, vacuum, and mixed build-plus-insert scans.
- Prove partially live groups do not score deleted lanes or emit invisible heap
  TIDs.
- Decide whether sparse inserted tails are rebuilt in-place, left until VACUUM,
  or delegated to Task 114's broader lifecycle/repack policy.

### Phase 5 - Evidence Packet

Run an `ecaz bench suite` matrix with packet-local artifacts:

- at least 10k, 50k, and 100k,
- 1M if smaller scales show a material storage or latency difference,
- source/f32 baseline plus the currently best compact rerank candidate,
- build-only, insert-heavy, and mixed build-plus-insert surfaces where feasible,
- warm and cold labels when cold I/O is part of the claim.

## Acceptance Criteria

1. Insert-side index rerank groups are no longer one-wide under normal bulk
   insert workloads.
2. Build and insert use one common logical grouping architecture, or the packet
   explains the remaining split and why it is required.
3. Continuation segments do not repeat full group metadata.
4. Scan retains direct lookup and does not rebuild a full sidecar map per query.
5. Correctness coverage includes insert, update, delete, vacuum, and mixed
   build-plus-insert scans on PG18.
6. Packet-local evidence reports group-width histogram, group count, storage
   bytes, recall, and latency.
7. The final packet recommends promote, iterate, or defer with benchmark-backed
   rationale.

## Evidence Requirements

Review packets must include:

- suite config and command lines,
- head SHA and format version,
- rerank format and placement,
- scorer width and logical group width,
- group count and group-width histogram,
- metadata bytes vs payload bytes,
- continuation segment count and average payload utilization,
- recall@10/NDCG@10,
- p50/p95/p99 latency and cache-state labels,
- index size and build/insert timing,
- mutation test logs.

## Dependencies and Coordination

- Follows Task 111h's rerank closeout and reviewer residual notes.
- Coordinates with Task 114 for broader dense-block lifecycle and repack policy.
- Coordinates with Task 42/NFR-016 for any persisted format-version change.
- Future compact formats, including RQ8/TQ/f16 revival work, should consume this
  shared layout instead of adding format-specific storage paths.
