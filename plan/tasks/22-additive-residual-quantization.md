# Task 22: Additive / Residual Quantization Feasibility

Status: proposed — **evaluate, do not auto-adopt**. Decision-gated.

Executes ADR-037.

## Scope

Investigate Additive Quantization (AQ) and Residual Vector Quantization
(RVQ) as successors to grouped PQ in the PqFastScan build path.

Target promise: ~2x compression at the same recall ceiling — a vector
encoded as the sum of M codebook entries, each from a small codebook, so
M × log2(K) bits total. For 1536-dim at similar recall to current 384 B
PQ4, AQ might fit in 128–192 B.

**This task is gated.** It has three decision points. The outcome may
be "adopt", "adopt with caveats", "keep researching", or "shelve". Write
the task to accommodate all four.

## Why not a blind port

Three open risks, none of which apply to OPQ or LSQ:

1. **FastScan compatibility.** AQ codes sum to reconstruct the vector,
   so scoring is `<q, sum(C_i[idx_i])>` = `sum(<q, C_i[idx_i]>)`. That
   factors into M separate LUT lookups, just like PQ. But the
   per-subvector LUT is over the full dim, not a 16-dim subblock, so
   LUT memory grows. Open: does FastScan's 16-byte LUT tiling trick
   still apply, or does AQ force a different scoring kernel shape?
2. **Encoding cost.** AQ assignment is NP-hard; practical encoders use
   beam search over M positions. Typically 10–100x slower than PQ
   encode. May cripple insert throughput on live indexes.
3. **Recall cliff.** AQ's theoretical ceiling beats PQ, but in
   practice the recall curve is flatter and drops off sharply near the
   frontier. Validate on the 50k warm real seam before trusting the
   paper's numbers.

## Subtasks

### Phase 1 — offline feasibility

- [ ] **Reference implementation.** `src/bin/aq_feasibility.rs`.
  Standalone binary. Trains AQ and/or RVQ codebooks on a training
  sample; encodes the full corpus; runs the 50k query seam with an
  offline scorer (no scan-path integration yet).
- [ ] **Compression at equal recall.** Plot AQ / RVQ / PQ curves:
  bytes per vector on the x-axis, recall@10 on the y-axis, on both
  1536 and 768 dim seams. Decision point: does AQ or RVQ beat PQ at
  the same recall by ≥30% in byte budget?
- [ ] **Encoding cost.** Measure per-vector encode time at target
  recall settings for each method. Report as a ratio against current
  PQ encode time.

**Decision gate 1:** If no method beats PQ's byte budget meaningfully
*and* no method shows acceptable encode cost, **shelve** the task and
record the null result in ADR-037.

### Phase 2 — scoring kernel investigation

- [ ] **LUT sizing.** Work out the memory footprint of the scoring
  LUT for AQ and RVQ at practical (M, K) settings. Compare to the
  96 KB footprint that already stresses Graviton L1D on TurboQuant.
- [ ] **FastScan-compatible RVQ.** RVQ (hierarchical residual PQ) is
  LUT-compatible by construction. Prototype an RVQ scorer that
  reuses the existing FastScan 16-byte LUT tiling — it's the most
  likely path to preserve our current kernel.
- [ ] **Full AQ scorer.** If RVQ-over-FastScan works, full AQ is an
  incremental upgrade. If not, a full AQ scorer is a new kernel and
  adds risk.

**Decision gate 2:** If neither AQ nor RVQ fits the FastScan LUT
shape and a new scoring kernel would be required, downgrade priority
to "research track" and handoff to ADR-037 as `RESEARCH` rather than
`ADOPT`.

### Phase 3 — insert path integration

- [ ] **Online encoder.** Port the beam-search encoder to the
  `aminsert` path. Budget: encode cost ≤3x current PQ encode, or
  the task shelves.
- [ ] **Insert throughput benchmark.** Compare PqFastScan insert
  throughput under PQ vs AQ/RVQ encoders at the same recall.

**Decision gate 3:** If insert throughput regresses by more than 3x
at equal recall, either (a) accept as a "bulk-load only" format and
reject live inserts, or (b) shelve. Record the choice in ADR-037.

### Phase 4 — if adopted

- [ ] **Wire format.** New format tag `INDEX_FORMAT_V3_ADDITIVE` or
  similar. Reloption `storage_format='aq'`.
- [ ] **Build path.** Training, encode, flush.
- [ ] **Scan path.** LUT prep, scorer, rerank.
- [ ] **Insert path.** Handle either the "bulk-load only" or live
  outcome from decision gate 3.
- [ ] **Vacuum path.** Page-layout compatible with PqFastScan's
  hot/cold split where possible; otherwise a separate vacuum
  implementation.
- [ ] **Measurement packet.** Recall, latency, size, build time,
  insert throughput on 50k warm real seam + 1M scale seam.

## Owns

- ADR-037
- `src/bin/aq_feasibility.rs` (new)
- Any new scoring kernel if Phase 2 requires one

## Dependencies

- Task 20 (OPQ). OPQ validates the rotation+codebook pipeline end to
  end; AQ/RVQ ride on top. Also, OPQ may do enough on the recall
  curve that AQ's incremental win isn't worth the structural cost.
- Task 15 stable PqFastScan format (the reference baseline).

## Unblocks

- If Phase 4 lands: ~2x index footprint reduction at equal recall,
  which compounds through DiskANN and SPANN replication (4x storage
  reduction end-to-end at billion scale).
- If shelved cleanly: an explicit null result in ADR-037, which
  frees reviewer attention for other frontier ideas.

## Out of scope

- LSQ (task 23). LSQ is a codebook-refinement trick orthogonal to
  AQ vs PQ.
- OPQ rotation (task 20).
- Wire format v3 bump for non-AQ reasons.

## Notes

- **AQ vs RVQ.** AQ is more general but harder to encode. RVQ is a
  specialization with hierarchical residuals and is easier to fit
  into FastScan's LUT model. Pragmatic ordering: RVQ first (lower
  risk), AQ second if RVQ looks promising.
- **This is the most speculative task in the queue.** Treat each
  decision gate as a real gate. Shelving cleanly is a valid outcome.
- **Do not merge a half-adopted AQ/RVQ.** If any phase fails, stop
  and write up the null result. A partial format landing is worse
  than no format landing.
