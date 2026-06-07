# IVF TQ+ Task-Local Format Plan

Task: 86

Packet: `reviews/task-86/011-ivf-tqplus-real-spread/`

Implementation commits:

- Benchmarked implementation: `e0ae9fe7dbcfb335cdaa7f47072416e5287ce5a4`
- Production-naming/validation cleanup: follow-up commit in this branch; no
  scoring-behavior change from the benchmarked implementation.

## Why A Plan Is Required

Task 86 says that any format-changing slice requires an ADR or task-local
format-version plan before landing. The IVF TQ+ measurement profile adds a new
SQL-visible IVF `storage_format` value, `turboquant_tqplus`, and persists a
small calibration model. That is a durable on-disk behavior change even though
the hot per-row posting payload stays the same size as TurboQuant.

This file is the task-local plan for that change.

## Format Delta

The TQ+ profile changes only IVF indexes whose metadata page records:

```text
storage_format = TurboQuantTqPlus
storage_format byte = 4
```

Existing values remain unchanged:

```text
0 = auto
1 = turboquant
2 = pq_fastscan
3 = rabitq
```

For `turboquant_tqplus`:

- each posting payload is the same packed 4-bit MSE code length used by
  TurboQuant no-QJL 4-bit;
- each posting `gamma` field stores the TQ+ per-vector renorm scalar;
- `metadata.pq_codebook_head` points to a two-tuple calibration chain;
- tuple group `0` stores `shift[dim]`;
- tuple group `1` stores `scale[dim]`;
- `metadata.pq_group_size = 0`.

The reuse of `IvfPqCodebookTuple` is deliberate for this measurement slice
because it already provides a WAL-covered float-vector model tuple chain. It is
not intended to rename TQ+ calibration as PQ codebooks in the long-term API.

## Compatibility

- Old indexes with storage-format tags `0..3` are decoded exactly as before.
- A pre-TQ+ binary that does not know tag `4` will reject a TQ+ IVF index as an
  unknown storage format. It will not reinterpret the bytes as TurboQuant.
- A TQ+ binary rejects TQ+ metadata if the calibration head is missing, if the
  chain does not contain exactly shift then scale, or if either tuple has the
  wrong group index.
- A TQ+ binary rejects calibration with wrong dimension lengths, non-finite
  shifts, non-finite scales, or zero scales.
- TQ+ currently requires the no-QJL 4-bit TurboQuant lane. Other bit widths or
  QJL-enabled dimensions must reject at build/query/insert time.

## Insert, Scan, And Vacuum Semantics

- Build: staged heap tuples initially keep the source vector; after calibration
  is trained, postings are re-encoded using the persisted TQ+ model.
- Insert: trained-index inserts load the persisted TQ+ model and re-encode the
  incoming source vector before writing the posting.
- Scan: query prep loads the persisted model once per scan opaque and folds
  scale/shift into the prepared LUT and bias.
- Vacuum: posting payload length remains the TurboQuant 4-bit MSE length, so
  existing payload-length driven posting walks still apply.

## Promotion Requirements

Before `turboquant_tqplus` should be treated as a general production format
rather than a measured IVF profile, do the following:

1. Decide whether to keep the reused `IvfPqCodebookTuple` chain or introduce a
   dedicated IVF calibration tuple kind.
2. Add mixed-version upgrade documentation for storage-format tag `4`.
3. Add a corruption/negative metadata fixture that proves missing calibration,
   wrong group order, trailing tuple, non-finite shift, and zero scale all
   reject.
4. Re-run the real10k/50k/100k TQ+ suite after any tuple-kind change.
5. Only then consider cross-AM TQ+ storage surfaces for SPIRE, HNSW, or
   DiskANN.

## Current Decision

Packet 011 is acceptable as an IVF measurement and review candidate because the
format is isolated by a new storage-format tag, old formats are unchanged, and
the benchmark evidence shows a real-corpus recall/latency win at unchanged hot
posting bytes. Promotion to a broader production API remains a follow-up
decision, not an implied outcome of this packet.
