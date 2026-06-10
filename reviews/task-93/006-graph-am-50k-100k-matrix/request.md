# Task 93 Packet 006: Graph-AM 50k/100k Measurement Matrix

Pure measurement packet (no code change): extends the HNSW and DiskANN
RaBitQ kernel cells from real10k (packet 004) to real50k and real100k,
completing every (AM × corpus) cell this host can measure for the Task 93
matrix.

## Results (full numbers in `artifacts/manifest.md`)

- **Recall byte-equal at all four new cells** (hnsw 0.8812/0.8906, diskann
  0.9917/0.9781 — identical between kernel-on and kernel-off).
- **Full `isa=neon` coverage**: 63k–73k candidates per cell, zero scalar
  spill, zero rows in kernel-off cells.
- **Scoring share**: 137–267 ns/candidate on the NEON path — every cell
  ≥2× against the packet-002 forced-scalar reference (2.9×–5.8×).
- **End-to-end**: the suite's apparent 100k kernel-on regressions invert
  under interleaved 64-iteration rechecks (hnsw on 3.62/3.66 vs off
  3.77/4.24 ms p50; diskann on 3.13 vs off 3.94 ms p50) — parity-or-faster,
  consistent with the packet-004 finding that run-order drift on this host
  exceeds cell deltas.

## Matrix status after this packet

M5-measurable cells are complete: IVF/HNSW/DiskANN × real10k/50k/100k ×
{scalar reference, NEON kernel}, recall byte-equal everywhere. Remaining
Task 93 columns are host-gated per packet 005: SVE (Graviton lane, AWS
authorization pending) and AVX2 (Intel desktop lane). The closeout matrix
packet will aggregate once either lane reports, or close with documented
deferrals if the lanes are descoped.

## Review request

Please review the cell evidence and the recheck methodology. No code to
review in this packet.
