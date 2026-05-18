# Review Request: C1 Task16 TurboQuant V3 Runtime Wiring

Current head at execution: `851da10`

## Context

Packet `427` landed the dormant TurboQuant V3 hot/cold page substrate:

- `TqTurboHotTuple` for graph-hot fields
- `TqRerankTuple` for cold scalar-code payloads
- metadata/versioning support for `INDEX_FORMAT_V3_TURBO_HOT_COLD`

That packet explicitly did **not** wire the new layout into build, insert,
scan, vacuum, or the debug/test readers. This slice does that runtime/build
integration so the V3 layout is now a real end-to-end storage format instead of
an unused page helper.

## What Landed

### Build path

- TurboQuant build output now writes V3 metadata instead of the old V1 scalar
  layout.
- Each built TurboQuant node now emits:
  - one neighbor tuple
  - one cold rerank tuple containing `(gamma, code)`
  - one turbo-hot tuple containing level / heap tids / neighbor tid /
    rerank tid / persisted binary sidecar words

### Graph/read path

- `GraphStorageDescriptor` now recognizes TurboQuant V3 as a distinct
  hot/cold storage layout with:
  - binary-sidecar width
  - rerank code length
- graph readers can now load exact TurboQuant elements by joining hot tuples to
  their cold rerank payloads
- layer-search / refill helpers now operate on storage descriptors rather than
  assuming scalar inline payloads

### Live insert / repair path

- live insert now supports TurboQuant V3:
  - empty-index metadata bootstrap chooses V3 for TurboQuant
  - duplicate detection reads hot tuples plus cold rerank payloads
  - append writes neighbor + rerank + hot tuples with stable cross-links
  - duplicate heap-tid coalescing updates hot tuples in place
  - invalid or deleted entry-point repair now works against V3 graph reads
- vacuum now supports TurboQuant V3:
  - pass 1 heap-tid compaction
  - repair request discovery
  - linear replacement-candidate collection
  - finalize/deleted marking

### Scan/debug/test path

- scan exact-score fallback now works against V3 TurboQuant elements by loading
  the cold rerank payload instead of assuming inline scalar code bytes
- TurboQuant binary live-rerank logic accepts both legacy scalar TurboQuant and
  V3 hot/cold TurboQuant storage
- debug helpers and page-decode test utilities now understand both TurboQuant
  tuple tags:
  - legacy `TQ_ELEMENT_TAG`
  - V3 `TQ_TURBO_HOT_TAG`
- rollover/tail-page tests were updated for hot/rerank/neighbor triplets, and
  their dimension search now uses cheap shape math instead of building a fresh
  quantizer per candidate dimension

## Validation

Green on this head:

- `cargo test`
- `bash scripts/run_pgrx_pg17_test.sh`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

## Readout

### 1. Packet `427` is no longer dormant

TurboQuant V3 is now exercised by the real AM lifecycle:

- build writes it
- insert mutates it
- scan reads it
- vacuum repairs it

### 2. This slice is storage/runtime plumbing, not a measurement claim

The packet intentionally does **not** attach new latency or recall claims yet.
It makes the V3 hot/cold layout real so the next packet can measure the serious
`50k, m=16, ef=128` lane on the new storage shape.

### 3. The next step is measurement, not more page-layout scaffolding

With the wiring in place, the next task-16 packet should:

- rebuild the isolated source-backed TurboQuant lane on V3
- capture warm SQL latency + internal stage profile
- compare against packet `426`'s quantized/heap-f32 decision point
