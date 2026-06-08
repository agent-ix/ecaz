# Task 87 Packet 001 Artifact Manifest

- head SHA: `d198913b7`
- task bucket: `reviews/task-87`
- packet path: `reviews/task-87/001-phase1-design`
- lane: Phase 1 design
- fixture: source review only
- storage format: TurboQuant no-QJL 4-bit target; no storage change in this
  packet
- rerank mode: not applicable
- command used: source inspection with `rg` and `sed`
- timestamp: 2026-06-08
- surface mode: no benchmark run; future per-AM measurement must use isolated
  one-index-per-table suite surfaces

## Artifacts

- `candidate-batch-design.md` — CandidateBatch design, quantizer walkthrough,
  per-AM mapping, pgvectorscale comparison, streaming contract, and measurement
  methodology.

## Key Result Lines Cited By Request

- `CandidateBatch` is a safe borrowed candidate-view abstraction under
  `src/am/common/`, not a quantizer-specific payload owner.
- The contract can host TQ 2-bit, TQ QJL, RaBitQ, binary fingerprint, and
  PQ/grouped-PQ without redesign.
- The 32-vector u8 nibble LUT kernel is deferred until after per-AM batch
  integration proves useful batch sizes.
- DiskANN currently needs explicit reviewer attention because this checkout
  exposes grouped-PQ and RaBitQ search codecs, not a TurboQuant search codec.
