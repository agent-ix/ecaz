## Feedback: ADR-030 v2 Design Checkpoint

Read this packet alongside the code in `spec/adr/ADR-030-fastscan-grouped-subvector-scoring.md`
and the follow-on packets 311-328. Summary across the whole v2 lane below; this note is the
anchor, individual packets carry shorter, more specific feedback.

### What looks right

- The design doc correctly names the three layers that have to move together: transform
  (SRHT), approximate scorer (grouped PQ4 FastScan), and rerank (scalar quantized). Each one
  now has a versioned metadata slot (packet 312) instead of being implicit.
- Decision to ship v2 behind `INDEX_FORMAT_V2_GROUPED = 2` with a separate build gate
  (`TQVECTOR_EXPERIMENTAL_ADR030_V2_BUILD`) is the right risk posture. Packet 323 adds an
  explicit scan-side rejection (`ADR030_GROUPED_V2_SCAN_UNSUPPORTED`) so a built v2 index
  cannot be silently mis-scored.
- Splitting hot tuple from cold rerank tuple is cleanly reflected in the tag space
  (`TQ_GROUPED_HOT_TAG = 0x03`, `TQ_RERANK_TAG = 0x04`) and in the two tuple types in
  `page.rs`. That keeps ADR-030 orthogonal to the ADR-031 binary sidecar rather than
  reopening it.

### What the 311 measurements actually justify

Packet 311 reports `spearman_rho = 0.8859` at `group_size = 16` with a `15.5x` speed
multiplier over exact scoring on SRHT-transformed 1536-dim vectors. That's the
load-bearing result for the whole lane: without it, the two-stage grouped-FastScan +
rerank pipeline is not clearly better than the scalar path it replaces.

Two things to be careful about as the scorer lands:

1. The 311 numbers are on pre-transformed vectors, in-process, without real page IO.
   The runtime win is going to be smaller than `15.5x` because (a) the binary prefilter
   already eliminates most candidates, and (b) the grouped score will be run inside a
   cache-warm inner loop where IR, not arithmetic, is the bottleneck on a lot of
   workloads. Plan to re-measure end-to-end against the exact+binary baseline before
   enabling v2 outside the build gate.
2. The approximate ranking is only "good enough" if rerank actually catches its
   mistakes. The rerank codec slot exists in metadata, but there's no end-to-end recall
   check in any packet yet. That's the biggest missing piece in the current slicing.

### Strategic note

The incremental slicing (311 feasibility → 312 metadata → 313 tuple contract → 314
pages → 315-319 build → 320-322 guarded write-out → 323 runtime gate → 324-328 read-side
seams) is a model for how to do this kind of cross-cutting change safely. Each packet is
tight, individually tested, and does not enable runtime until scoring is wired.

Keep doing this. Do not collapse the remaining read-side work into a single "enable
grouped runtime" packet.

### What the next slices need to address

- Insert path (`src/am/insert.rs`) has no format-version guard. Today it is only
  protected by `build_source_column` being unset, which is fragile. Before the build gate
  is loosened, the insert path must either reject grouped-v2 indexes explicitly or
  produce grouped-v2 tuples of its own.
- Vacuum path (`src/am/vacuum.rs`) decodes via `TqElementTuple::decode` with no
  grouped-v2 awareness. A vacuum of a v2 index today would mis-decode tuples.
- Rerank fetch. No packet yet touches cold rerank tuple reads. That's the missing half
  of the hot-cold split.

None of these blocks merging the current packets. They block lifting the experimental
gate.
