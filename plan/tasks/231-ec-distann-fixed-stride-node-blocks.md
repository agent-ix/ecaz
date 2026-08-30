# Task 231: ec_distann Fixed-Stride Graph/Vector Blocks

Status: **implementation complete through lifecycle/DML — Packet 003 reader
and Packet 004 raw-tail allocator are review-closed DONE; Packet 005
full-scale preregistration seq-03 is review-open and no measurement has run;
Packets 001/002 remain open for final design/handoff reconciliation**
(2026-08-30). Priority: P1 graph-storage/retrieval latency.

Program ledger: `plan/design/ec-distann-recall-latency-roadmap.md`, candidate
ARCH-17. This is the DiskANN/DistributedANN-style whole-node retrieval
experiment and remains isolated from the row-columnar Task 232.

## Why

The current physical generation stores graph nodes as PostgreSQL heap tuples
containing a bytea record and resolves vec_id through a B-tree to a heap TID;
the exact vector lives in a separate row-tier tuple. That layout is robust and
transactional, but a search expands whole graph nodes, not individual graph
fields. It therefore pays tuple/index/detoast work around a naturally
fixed-shape access unit.

DiskANN stores each node as a directly addressable fixed-length disk record
containing vector and adjacency, packs smaller nodes into blocks, and aligns
multi-block nodes so the address follows from node id. DistributedANN likewise
uses graph-node entries as the key-value read unit and duplicates compressed
neighbor representations into those entries to avoid secondary reads. ecaz
already duplicates neighbor codes; this task tests whether matching the
physical unit to the whole-node read improves PostgreSQL-backed serving.

## Goal

Implement and benchmark an opt-in relation-backed fixed-stride node extent:
owner-local dense ordinal to a bounded contiguous record holding graph header,
exact vector, search code, neighbor ids, and neighbor codes. Prove whether
direct block arithmetic and one bounded whole-node read outperform the current
B-tree + graph heap + row-tier vector path.

## Entry conditions

1. Tasks 222--224 define the unstacked production control.
2. Tasks 229 and 230 have completed, but neither layout is enabled in this arm.
3. The design packet freezes node-size arithmetic for all supported dimensions,
   graph degrees, and codecs before implementation.

## Required implementation

### P1 — Relation-backed block format

- Use PostgreSQL-managed relation storage and WAL; do not introduce an external
  raw sidecar file.
- Map each vec_id through a generation-local directory to an owner-local dense
  ordinal. The ordinal determines the aligned block/offset without a graph-heap
  CTID.
- Store a versioned fixed header, full-precision exact vector, search code,
  neighbor count/ids/codes, and canonical padding in one logical node extent.
- Pack multiple nodes per PostgreSQL block when they fit. When a node exceeds
  `BLCKSZ`, align it to the first block of a fixed-count multi-block extent and
  validate every segment before decode.
- Keep source payload materialization on the ordinary row-tier control so the
  A/B isolates graph/vector storage rather than also testing columnar payloads.

### P2 — Reads, lifecycle, and DML

- Batch requested ordinals, read bounded extents, and decode into reusable
  buffers without SPI or heap tuple deformation.
- Preserve exact owner placement, tombstones, ordered response identity,
  Algorithm-1 limits, and gateway-copy behavior.
- Make build/handoff, digest, publication, restart, retention, retirement,
  reclaim, and rollback aware of the block relation and directory format.
- Define append extents and atomic directory publication for Task 167 inserts
  and replacements; no in-place rewrite of a Published base node without the
  existing transaction/fencing contract.

### P3 — Evidence

- Test boundary sizes: several nodes per block, exactly one block, and
  multi-block extents; truncated, swapped-version, bad-padding, wrong-ordinal,
  and digest corruption must fail closed.
- Run isolated current-heap versus fixed-stride A/B at 10k, 50k, and 100k using
  `ecaz bench suite`, with both the standard warm protocol and a suite-driven
  controlled-residency/cold-read profile where available. Extend the suite
  runner first if that profile is missing.
- Compare arms at matched fixture position, or use a preregistered
  counterbalanced envelope that separates position/warmth from the candidate.
  Never compare a fresh-build control only against a reused candidate.
- Report directory probes, graph/vector block reads, bytes requested/read,
  buffer hits, decode/score time, traversal rounds, owner/transport work,
  latency/tails, recall, build/DML time, storage padding, and conformance.

## Decision rule

The fixed-stride prototype and full 10k/50k/100k matrix are mandatory. Warm
and controlled-residency results must be reported separately; a cold-read win
cannot be represented as a warm-path win. Continue to Task 232 regardless and
retain the opt-in prototype through Task 233's mandatory factorial hybrid
experiment even if this isolated arm closes STOP.

## Non-goals

- Columnar payload storage or selective per-attribute reads.
- Graph/community reordering, BFS page clustering, prefetch, or a changed
  search budget; those would confound the layout A/B.
- External files or mmap that bypass PostgreSQL relation/WAL ownership.
- Combining Tasks 229 or 230 into the candidate arm.
- Combining the Task-232 payload tier into this attribution arm; Task 233 owns
  that composition after both isolated mechanisms have reported.

## Acceptance

1. Direct ordinal/block arithmetic and all page-fit cases are format-tested.
2. Whole-node reads preserve byte-identical ordered results and fail-closed
   lifecycle semantics.
3. Task 167 mutation behavior works through a bounded append/overlay contract.
4. Full-scale recall, latency, storage, build, and DML evidence supports a
   reviewed PROMOTE or STOP decision.

## Required review packets

1. `reviews/task-231/001-plan/`
2. `reviews/task-231/002-hybrid-handoff/`
3. `reviews/task-231/003-format-and-reader/`
4. `reviews/task-231/004-lifecycle-and-dml/`
5. `reviews/task-231/005-full-scale-decision/`

## References

- Tasks 168, 179, 204, 222--224, and 233
- Microsoft DiskANN `disk_index_writer` fixed block layout
- `DISTRIBUTEDANN` §2.1--2.3 (arXiv:2509.06046)
- FR-076, FR-079, FR-082, FR-083
- NFR-016, NFR-018, NFR-021, NFR-022
