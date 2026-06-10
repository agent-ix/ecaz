# Task 93 Packet 004: HNSW + DiskANN Runtime Batch Accumulation

Completes acceptance criterion 2 for the graph AMs: HNSW and DiskANN RaBitQ
scan scoring now routes through `CandidateBatch` + the rabitq32 kernel at
runtime, with kernel-on/off bench cells per AM.

## Commits under review

- `4b382584d` — HNSW non-binary traversal arm + DiskANN prefilter batch arm
  (+ new `ec_diskann.candidate_batch_scoring` GUC, default on) + DiskANN
  routing-proof test.
- `1516996f8` — HNSW binary-branch fix: default RaBitQ scans run with the
  binary prefilter active, so the batching had to live on the binary
  branch's survivor loop (run 1 of the bench caught this: zero hnsw rows).
- `23f618aa5` — partial-width SIMD dispatch (the substantive design change
  in this packet, below).
- `869645f17` — Task 100 stub (`ec_ivf` plain-scan planner guard), filed per
  the packet-003 reviewer follow-up.

## Design change to review: partial-width SIMD dispatch

Bench run 2 measured what the 32-wide gate means for graph AMs: HNSW
survivor batches average ~22 candidates and DiskANN node batches ~10, so
the "sub-32 → scalar tail" rule sent essentially 100% of graph-AM
candidates through the forced-scalar path and regressed default-on latency
(HNSW p50 +37%, DiskANN +25%). The task plan's width gate is a means (block
amortization), not an end; acceptance criterion 5 (no end-to-end
regression) has to win.

`score_rabitq_bits1_partial` (1..=31 candidates) now dispatches sub-width
runs and block tails through the best backend: the NEON implementation
scores pairs via the production pair primitive plus the single-candidate
primitive for an odd trailing code — the identical operation orders as
production `estimate_ip_bits1_batch`, proven bit-equal in
`partial_dispatch_matches_anchor_and_production_batch`. The scalar backend
keeps the forced-scalar anchor order, so scalar-host behavior and the
strict parity contract are unchanged.

**Counter-semantics consequence (please confirm):** `kernel_*` rows now
mean "SIMD-backend flushes" (full blocks and partial runs) and `scalar_*`
means strictly scalar-executed work. The alternative — recording
NEON-executed partial runs as `scalar_*` — would misattribute the ISA; and
keeping forced-scalar tails just to preserve the old reading is exactly
the regression run 2 measured. Flagging for ADR-076/ADR-077 wording at
Task 99.

## Validation

Logs in `artifacts/`; all at HEAD `23f618aa5`: clippy `-D warnings` clean;
rabitq32 6/6, candidate_batch 10/10, ec_ivf 27/27, ec_diskann 13/13,
ec_hnsw 81/81. Counter/routing tests are partial-aware and ISA-aware.

## Bench evidence (local M5, PG18, `ecaz bench suite` + recheck runs)

Full numbers and the run-1/run-2/run-3 development history in
`artifacts/manifest.md`:

- **Recall byte-equal** on both AMs (HNSW 0.9422, DiskANN 0.9984, identical
  between cells).
- **Full SIMD coverage**: `surface=hnsw` 66,961/66,961 and
  `surface=diskann` 39,353/39,353 candidates on `isa=neon` kernel rows;
  zero rows in kernel-off cells.
- **Scoring share**: 230 ns/cand (HNSW) and 285 ns/cand (DiskANN) vs the
  packet-002 forced-scalar reference (793 ns/cand, IVF surface) — ≥2×
  with the cross-surface caveat noted in the manifest.
- **End-to-end**: parity within machine noise on both AMs after the
  partial-width fix (interleaved recheck pairs included as artifacts;
  run-order drift on this host exceeds the cell deltas).

## Review request

Please review the two HNSW accumulation sites (non-binary and
binary-survivor arms), the DiskANN batch arm + GUC, the partial-width
dispatch and its counter-semantics shift, and the bench evidence. Remaining
Task 93 work after this packet: SVE backend (Graviton lane), AVX2 backend
(Intel lane), per-(AM × ISA) closeout matrix.
