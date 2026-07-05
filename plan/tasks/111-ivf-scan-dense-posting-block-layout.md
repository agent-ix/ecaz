# Task 111: IVF Scan-Dense Posting Block Layout

Status: **proposed**.
Priority: P0 latency.

## Goal

Reduce `ec_ivf` scan latency by changing the build-time posting layout from
row-shaped posting tuples into page-local, scan-dense blocks that can be scored
directly by the existing batch scorer.

The primary target is lower p50/p95/p99 at unchanged recall and without
storing full source vectors in the index. TurboQuant and RaBitQ are both in
scope because they are the active compact IVF scan formats.

## Why

Current IVF posting pages store one logical posting tuple at a time:

- list id,
- delete state,
- heap TID payload,
- gamma / scalar metadata,
- rerank TID,
- compact quantized payload.

The scan path then decodes those tuples and copies their fields into
scan-local structure-of-arrays scratch buffers before batch scoring. That
keeps the on-disk format simple for inserts and deletes, but it means the hot
path pays tuple decode, branch, line-pointer, and reshaping costs before the
SIMD/batch scorer can do useful work.

This task tests the opposite layout for immutable build-time data: write
posting pages in the same shape the scorer consumes.

## Scope

- IVF only.
- Frozen build-time posting blocks only.
- TurboQuant and RaBitQ compact payloads.
- Reloption-gated or format-version-gated layout.
- Existing row-shaped posting layout remains supported for old indexes and
  mutable/delta postings.
- Scan must support mixed frozen dense blocks plus row-shaped postings.
- No full-vector sidecar storage.

## Non-Goals

- Do not implement residual quantization in this task.
- Do not change heap-f32 rerank policy.
- Do not change nprobe, centroid routing, or adaptive probing behavior.
- Do not make the new layout the default without benchmark evidence and an
  explicit promotion decision.
- Do not change SPIRE, HNSW, or DiskANN layout in this task.

## Proposed Layout

The exact page format is part of Phase 1, but the candidate shape is:

```text
IvfFrozenPostingBlockV2 {
  header:
    format tag
    list id or list-span descriptor
    count
    payload stride
    live/dead bitmap, if needed for later lifecycle compatibility

  arrays:
    gamma[count]
    heap_tid_count[count]
    heap_tid_offset[count]
    heap_tids[total_heap_tids]
    rerank_tid[count]
    payloads[count * payload_stride]
}
```

The scan path should be able to pass `gamma[]` and `payloads[]` directly to
the batch scorer without first reconstructing equivalent vectors in scratch.

## Phases

### Phase 1 - Layout Design and Cost Audit

- Document the current row-posting tuple layout and scan-time reshaping path.
- Add or reuse counters for posting pages read, postings visited, postings
  scored, scratch flushes, scratch copy bytes, and approximate-scan time.
- Define the frozen block tuple/page format, version gate, and old-index
  compatibility contract.
- Define how mixed old row postings and new frozen blocks are detected during
  scan.

Stop condition: if counters show posting decode/copy is not material at the
target high-recall cells, close with evidence and do not change the on-disk
format.

### Phase 2 - Frozen Block Writer

- Add a build-time writer for scan-dense frozen posting blocks.
- Keep insert/update paths on the existing row-shaped appendable layout.
- Preserve deterministic build output for the same seed, input data, and
  reloptions.
- Add static encode/decode invariants and focused page-format tests.

### Phase 3 - Direct Frozen Block Scan

- Add scan support for frozen dense blocks.
- Route TurboQuant and RaBitQ frozen blocks through the batch scorer without
  per-posting scratch reconstruction.
- Preserve candidate deduplication, deleted-row filtering, and heap TID
  expansion semantics.
- Keep old row-posting scan support live.

### Phase 4 - Mixed-Layout Compatibility

- Support indexes that contain frozen dense blocks plus row-shaped delta
  postings.
- Add compatibility smoke tests for old-format indexes.
- Add EXPLAIN/debug counters showing how many candidates came from dense
  blocks versus row postings.

### Phase 5 - Benchmark Gate

Run packet-backed same-host benchmarks for at least:

- real 100k and 1M where available,
- TurboQuant IVF,
- RaBitQ IVF,
- warm latency and recall,
- index size and build time,
- posting pages read and candidates scored.

Promotion criteria:

- Recall is unchanged for the same query set and reloptions.
- p50 improves materially at the target high-recall cells, with no p95/p99
  regression that erases the win.
- Index-size change is reported and justified.
- Build-time cost is reported.

## Acceptance Criteria

1. Dense frozen posting blocks are implemented behind a gate.
2. Existing row-shaped IVF indexes remain readable.
3. TurboQuant and RaBitQ scan paths pass focused correctness tests.
4. Mixed frozen dense and row/delta scans return the same candidates as the
   legacy path for controlled fixtures.
5. A benchmark packet reports latency, recall, build time, index size, posting
   pages, candidates, and scan counters.
6. The final packet explicitly recommends promote, iterate, or abandon.

## Evidence Requirements

Use `ecaz bench suite` for all benchmark matrices. Packet manifests must record:

- head SHA,
- reloptions,
- storage format,
- payload stride,
- layout gate,
- q-count,
- warmup policy,
- whether surfaces are isolated one-index-per-table fixtures,
- recall@10, NDCG@10, p50/p95/p99, mean,
- index size and build time,
- relevant scan counters.

## Dependencies and Coordination

- Builds on the IVF scan and batch-scoring work from Tasks 51, 87, 91, 92,
  93, 97, 99, and 105.
- Coordinate with Task 42 if a new durable page format version is introduced.
- Task 114 owns post-delete density, repack, and lifecycle policy; this task
  may include only the minimum lifecycle handling required for correctness.
