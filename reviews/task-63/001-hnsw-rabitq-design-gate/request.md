# Task 63 Review Request: HNSW RaBitQ Design Gate

## Summary

This is the Task 63 design-gate packet for adding `storage_format = 'rabitq'`
to `ec_hnsw`. It does not include implementation or benchmark output. It
records the implementation strategy unlocked by Task 64 packets 001 and 002,
and names the measurement evidence still required before treating RaBitQ HNSW
as ship-ready.

Current head:

- `d6ba599ae4e1ead130c79fa6596688b93db9529a`

## Recommendation

Proceed with a narrow RaBitQ implementation behind the HNSW-local codec adapter,
but keep it behind an evidence gate until matched recall/latency/storage data
exists.

The implementation should:

- keep the default HNSW storage format as `turboquant`;
- add `rabitq` as an explicit reloption value only;
- reuse shared `RaBitQQuantizer` for encode and prepared query scoring;
- store RaBitQ search codes in an HNSW hot graph tuple with no binary sidecar;
- keep a cold TurboQuant rerank payload for final quantized rerank parity;
- require a real source vector for RaBitQ build and insert.

## Traversal Viability

RaBitQ approximate scores can be wired into HNSW traversal with the same score
polarity convention used by existing HNSW grouped scoring: inner-product
estimates become negative distances for HNSW ordering.

This is still an empirical risk. The required evidence before ship-ready status
is recall@10 against TurboQuant and PqFastScan at matched `ef_search` values on
at least 50k and 100k rows.

## Rerank Strategy

First implementation should preserve a cold rerank payload rather than relying
only on RaBitQ estimates for final ordering. That keeps the first RaBitQ HNSW
version comparable to PqFastScan's hot/cold shape and limits recall risk during
the traversal experiment.

## Payload Layout

Use an HNSW-local RaBitQ hot/cold layout:

- hot tuple: heap TIDs, neighbor TID, rerank TID, zero binary sidecar words,
  RaBitQ search code bytes;
- cold tuple: existing `TqRerankTuple` with TurboQuant gamma/code;
- metadata: explicit RaBitQ codec kind and RaBitQ bit width in the metadata
  shape fields.

This keeps RaBitQ math in `src/quant` and HNSW persistence in `src/am/ec_hnsw`.

## Insert and Vacuum

Insert should be supported only when the insert path has access to the source
vector needed to encode RaBitQ. Indexed `ecvector` supplies that directly.
Indexed `tqvector` must require `build_source_column` or fail with a clear
error.

Vacuum should share the existing HNSW graph repair flow and retain the hot
tuple plus cold rerank payload. If RaBitQ vacuum parity cannot be completed in
the first implementation packet, the feature must reject vacuum-sensitive usage
explicitly rather than silently leaving partial cleanup.

## Measurement Gate

Before marking Task 63 ship-ready, create a benchmark packet driven by
`ecaz bench suite` with checked-in SuiteConfig. Required rows:

- formats: TurboQuant, PqFastScan, RaBitQ;
- sizes: 50k and 100k minimum;
- metrics: recall@10, p50/p95/p99 latency, build time, and index size;
- surface: one index per table, matched `m`, `ef_construction`, and
  `ef_search`.

1M data is optional unless the benchmark host is already available.

## Task 64 Dependency

Task 63 implementation should extend:

- `HnswStorageCodec` for reloption/metadata/sizing;
- `GraphStorageDescriptor` only for tuple-read layout;
- existing scan/insert/vacuum adapter hooks for scorer and retention behavior.

Do not introduce a shared cross-AM codec trait during the first HNSW RaBitQ
implementation. Document repeated DiskANN/HNSW method shapes for the later
ADR-071 extraction trigger.
