# Task 111h: IVF Persisted Rerank Format Sweep

Status: **proposed**.
Priority: P0 correctness of the coarse-rerank product decision.
Parent: `111e-ivf-coarse-rerank-candidate-pipeline.md`,
`111g-ivf-coarse-rerank-representations.md`.

## Goal

Replace the misleading 111g "table-side f16/rabitq4" experiment with a real
persisted-rerank architecture, then implement and benchmark the full rerank
format space before making any promote/abandon decision.

The decision must compare:

- existing source-vector f32 rerank,
- persisted f16 rerank,
- persisted RaBitQ-4 rerank,
- persisted RaBitQ-8 rerank,
- persisted TurboQuant rerank.

The task is not closeable until every format above is either implemented and
benchmarked or rejected by packet-local evidence showing a concrete correctness
or layout blocker. "Deferred because not implemented yet" is not an acceptable
exit state.

## Why

The 111g benchmark labels hid two important facts:

- `rerank_placement=table, rerank_format=f16` did **not** mean f16 was persisted
  in table storage. It fetched the existing f32 heap vector and converted the
  candidate source vector at query time. That saves no IO and adds per-row
  conversion work, so it is not a valid storage-format experiment.
- `rerank_placement=index, rerank_format=f16` persisted f16 bytes, but the
  sidecar layout stored large 2048-d f16 payloads in page-sized sidecar tuples.
  That collapses to poor page density, hash/map lookup and copy overhead, and
  f16 unpack before scoring. It does not test the theoretical packed f16 win.

The current source-vector f32 rerank is therefore the real baseline: it adds no
extra table or index payload beyond the existing source vector, has exact recall,
and is already fast in the warm-cache sweep. Any compact rerank representation
must beat that baseline on matched recall/latency, total storage, cold/remote IO,
or index-only serving value.

The 111g/005 direct-TID sidecar fix also changes the baseline interpretation:
index-side f16 is no longer assumed to be a `150 ms` directory-bound path on
fresh builds. Treat the current `0x2A` direct-TID sidecar as a legacy baseline
to measure against, not as the final compact-layout answer. The follow-up must
make clear which results are from source-vector rerank, diagnostic query-time
conversion, legacy `0x2A` sidecar, and the new packed rerank layout.

## Correct Placement Semantics

Use explicit placement names in code/docs/packets. Do not overload "table-side"
to mean query-time conversion from the source vector.

- `source`: use the existing indexed f32 source vector from the heap table.
  This is the current exact f32 baseline. It adds no rerank payload storage.
- `table`: persist a separate table-owned rerank payload. This can be a real
  table column, generated/stored column, companion table, TOAST-backed payload,
  or another documented PostgreSQL-owned storage design. It must not convert the
  source vector per candidate at query time.
- `index`: persist rerank payloads inside the IVF index. This must use a packed
  layout designed for scorer-width reads, not the 111g one-fat-sidecar-entry
  layout.

If the public reloption remains `rerank_placement=table`, its behavior must
match the persisted table-owned meaning above. Query-time f32-to-compact
conversion must be rejected or renamed as a diagnostic-only mode.

## Architecture Requirements

- Build one common rerank payload architecture. Do not add f16, RaBitQ-8, or
  TurboQuant one-offs.
- Define a narrow rerank payload codec interface for:
  - build/insert encoding from source f32,
  - query preparation,
  - payload byte length and alignment,
  - batch scoring from persisted payload bytes,
  - optional scalar fallback,
  - storage/accounting metadata.
- Reuse the existing `QuantCodec` / `candidate_batch` surfaces where they fit,
  but add a rerank-specific adapter if rerank payload access needs different
  metadata or scoring semantics.
- f16 must be persisted at build/insert time. Query time may convert the query
  once, but must not convert every candidate source vector from f32 to f16.
- f16 scoring should avoid unnecessary allocation and full f32 materialization.
  If the first implementation decodes to f32, benchmark it against a direct
  packed-f16 scorer and record the gap.
- Direct packed-payload scoring must report decode/materialization work. A path
  that allocates one `Vec<f32>` or equivalent full-width buffer per candidate is
  not sufficient evidence for rejecting the compact format.
- RaBitQ-8 and TurboQuant rerank are required implementations for this task.
  They must share the same placement/payload interface as f16 and RaBitQ-4.
- Build, insert, delete/vacuum, and rebuild paths must keep persisted rerank
  payloads consistent with source vectors.
- Any durable layout change must bump the IVF format version and update the
  upgrade matrix/fixtures per NFR-016 and Task 42.

## Index Layout Requirements

The current `0x2A` compact sidecar is not sufficient for the final index-side
format. Replace or supersede it with a layout that can actually test compact
rerank payload locality.

Required shape:

- Logical dense rerank group size equals the target scorer width, not page
  capacity.
- Segment tuples are physical fragments of that logical group.
- Flush only at logical group completion, list boundary, live-TID budget
  exhaustion, or final tail.
- Pad final partial groups to scorer width for scoring, but emit only valid,
  live, nondeleted postings.
- Store group metadata once, preferably in a header segment:
  - list id,
  - valid/live count,
  - live/deleted bitmap,
  - gammas/coarse scores,
  - heap TID counts/offsets,
  - rerank payload offsets/TIDs,
  - heap TIDs where needed.
- Continuation segments must be payload-heavy. They must not repeat full metadata
  arrays per page.
- Scan must resolve survivor payloads by direct group/segment offsets, not by
  rebuilding a heap-TID hash map per query.
- Scan should avoid owned per-survivor payload copies. Prefer page-local slices,
  bounded scratch arenas, or scorer APIs that can consume segment/group payload
  ranges directly. A path that copies each survivor into a `Vec<u8>` map entry
  and then copies again into a batch-scoring slab must be measured as a legacy
  baseline, not accepted as the final compact layout.
- The layout must report bytes/pages read for metadata and payload separately.

## Table Payload Requirements

If table-owned compact payloads are implemented, their storage and read path
must be real, measurable storage, not query-time conversion:

- define how payloads are stored and maintained on INSERT/UPDATE/DELETE,
- preserve MVCC/snapshot semantics,
- report table payload size separately from the source vector column,
- report heap/table blocks and bytes touched during rerank,
- compare against `source` f32 with the same candidate frontier and nprobe.

## Benchmark Matrix

All benchmark matrices must be driven by `ecaz bench suite` with checked-in
suite configs and packet-local artifacts.

Required dimensions:

- corpus: real 10k, 50k, 100k, and 1M,
- nprobe: `8, 16, 32, 64, 128, 200`,
- rerank_width / candidate_k: at least `32, 64, 128, 256`,
- placements:
  - `source` for f32 baseline,
  - `table` for persisted compact payloads when implemented,
  - `index` for persisted compact payloads,
- formats:
  - f32 source baseline,
  - f16,
  - RaBitQ-4,
  - RaBitQ-8,
  - TurboQuant.

Required metrics:

- recall@10 and NDCG@10,
- p50/p95/p99 and mean latency,
- build time,
- index size,
- table source-vector size,
- table compact-payload size,
- total storage,
- coarse candidates scanned,
- rerank candidates retained,
- payload bytes read,
- pages/blocks read by source/table/index placement,
- stage timings for coarse scan, payload fetch, decode, and rerank scoring.

Run warm-cache local sweeps and at least one cold-cache or remote-storage sweep
for the final candidate formats. Compact payloads are only compelling if they
win under the storage/cache conditions they are designed for.

## Checklist

- [x] Rename or document current `table` behavior so query-time compact
      conversion cannot be mistaken for persisted table storage.
- [x] Reject or gate diagnostic query-time f16/rabitq conversion paths outside
      the benchmark-only diagnostic surface.
- [x] Define the common rerank payload codec interface.
- [x] Implement persisted f16 payload encoding for build and insert.
- [x] Implement persisted RaBitQ-4 payload encoding under the same interface.
- [x] Implement persisted RaBitQ-8 payload encoding under the same interface.
- [x] Implement persisted TurboQuant payload encoding under the same interface.
- [x] Implement the packed index-side rerank group/segment layout.
- [ ] Benchmark the existing `0x2A` direct-TID sidecar path as a legacy
      index-side baseline before replacing or superseding it.
- [x] Implement table-owned persisted compact payload storage, or produce
      packet-local evidence explaining why PostgreSQL table-owned storage is not
      viable and what replaces it. Evidence:
      `reviews/task-111h/034-table-owned-storage-rationale/`.
- [x] Implement direct payload lookup without per-query heap-TID hash-map
      rebuilds for the index-side path.
- [ ] Implement or explicitly benchmark away owned per-survivor payload copies
      and double-copy batch-scoring slabs in the compact index path.
- [ ] Add EXPLAIN/admin/counter coverage for placement, format, payload bytes,
      pages read, decode time, and scoring time.
- [ ] Add PG18 correctness fixtures for create/insert/update/delete/vacuum,
      mixed old/new postings, and snapshot-visible rerank payloads.
- [x] Specifically cover live insert and vacuum for direct payload pointers,
      fallback directory/full-chain lookup, and mixed postings that cannot carry
      an unambiguous direct pointer.
- [x] Add encode/decode and scalar-vs-batch differential tests for every format.
- [x] Add a no-query-time-source-conversion regression test for persisted compact
      formats: source-vector f32 may be read only for the `source` baseline or
      an explicitly diagnostic mode.
- [ ] Run the full `ecaz bench suite` matrix at 10k/50k/100k/1M.
- [ ] Publish packet-local manifests and raw results for every suite.
- [ ] Produce a final decision table comparing f32 source, f16, RaBitQ-4,
      RaBitQ-8, and TurboQuant at matched recall.
- [ ] State promote / iterate / abandon for each format and placement with
      evidence. No format may be left as "not tried".

## Acceptance Criteria

1. The task removes the misleading query-time compact conversion interpretation
   from the product-facing `table` placement.
2. f16, RaBitQ-4, RaBitQ-8, and TurboQuant rerank payloads are implemented
   through one common architecture, or each non-implementation has packet-local
   proof of a concrete blocker.
3. Index-side persisted payloads use a scorer-width packed group/segment layout
   with payload-heavy continuation segments.
4. Table-owned persisted payloads are either implemented and measured or
   explicitly replaced by an evidence-backed storage design.
5. PG18 correctness coverage proves source/table/index payload consistency
   across build, insert, delete/vacuum, and rebuild.
6. The full benchmark matrix is packet-local, reproducible through
   `ecaz bench suite`, and includes recall, latency, storage, build, and
   read-amplification metrics.
7. The final packet makes an evidence-backed decision for every format and
   placement. Deferring RaBitQ-8, TurboQuant, or efficient f16 is not an
   acceptable closeout.

## Evidence Requirements

Each review packet must include:

- checked-in suite config,
- `suite-manifest.json`,
- `results.jsonl`,
- storage logs with table/index/total bytes,
- reloptions and effective placement/format,
- corpus manifest and row/query counts,
- head SHA,
- whether the run used isolated one-index-per-table or shared-table surfaces,
- key result lines cited by `request.md`.

Do not cite terminal scrollback, `/tmp`, or summary-only claims. If a result is
not in a packet-local artifact, it did not happen.
