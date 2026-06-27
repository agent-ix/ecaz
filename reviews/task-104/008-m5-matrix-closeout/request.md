# Task 104 packet 008 — M5 matrix deliverable + closeout (review request)

- Task: `plan/tasks/104-apple-silicon-m5-bench-optimization-lane.md` scope item 5 / acceptance criteria 1-6
- Branch: `task-104-m5-bench-optimization`
- Deliverable: `m5-index-quant-option-matrix.md` (this packet) — the
  Apple-silicon supported-target column for Task 99, with per-cell
  scoring-share, e2e deltas, recall, and kernel_status markers.

Acceptance criteria status:

1. Day-one parity/test suite green on M5 — packet 001 (after fixing the
   qjl32 NEON production-scorer alignment gap + two stale main tests).
2. Full index x quant x option matrix with kernel_status markers and a
   non-1536-dim QJL fixture — packets 002-007.
3. Kernel-on cells at isa=neon / scalar_candidates=0 with width
   histograms; recall byte-equal on all 40 measured on/off pairs;
   tolerance families documented (QJL 4-ulp contract) — packets 002-007 +
   matrix doc.
4. Every family >=1.5x floor or documented outcome — all PASS after the
   packet 007 qjl32 kernel rewrite (0.83x -> 3.2-3.5x); tiled_lut retired
   confirmed; structurally-absent cells marked (HNSW rabitq bits4/8,
   SPIRE PqFastScan, DiskANN TQ @1024).
5. Kernel code changes are on the branch awaiting review/merge to main
   BEFORE the G4 trip: `16133580a` (qjl32 prod-scorer alignment),
   `f88c640d3` (qjl32 NEON block kernel), `5c44d9f45` (NEON octet
   remainder routing), `d1235077c` (suite-runner retired marker).
   aarch64-only except the remainder-dispatch rename in candidate_batch
   (pure rename on the x86 path); no Intel re-run required.
6. Matrix citable from Task 99 — `m5-index-quant-option-matrix.md`.

Open findings handed to Task 99: SPIRE PqFastScan structurally absent
end-to-end (product gap); HNSW/DiskANN grouped-PQ batch engagement gaps
(Task 94/101 lane).

## 2026-06-11 update after packet 007 feedback seq 1

The qjl32 remainder band now routes through the NEON octet
(`5c44d9f45`); AC3/AC4 re-verified on the packet 007 octet-round
artifacts — no batch-side scalar fallback remains (residual `isa=scalar`
rows are the one-off path with empty width histograms, plus sub-8
flushes by cascade design), HNSW QJL e2e moved from neutral to
-17.4%/-13.5%, recall unchanged at every cell. The matrix doc gained an
attribution note spelling out the one-off-row semantics the reviewer
flagged.
