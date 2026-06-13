# Task 106 packet 001 — multi-bit RaBitQ kernel + M5 evidence

Status: review request (2026-06-13). Coder lane.

## Summary

Task 106 closed the multi-bit RaBitQ weak deferral and three of the four
named unified-driver gaps, with M5 evidence. This packet carries the
kernel microbench, the index-level bench, and the build/smoke results.

## Outcomes

- **Multi-bit RaBitQ (IVF bits=2/4) — closed + measured.** New rabitq32
  multi-bit block kernel (scalar/NEON/AVX2; SVE→NEON). M5 microbench +
  index-level bench decided the routing on evidence: bits=2 → block kernel
  (~3× win, no NeonBits2 exists), bits=4 → arithmetic estimator (NeonBits4
  beats the block kernel ~2.7×). AVX2 built but Intel-only.
- **Slice 1 SPIRE×RaBitQ** — migrated to the unified driver (code-correct,
  counter-capable); index-level A/B shows ~0% e2e (inherent to SPIRE).
- **Slice 3 IVF×TQ-QJL** — root-caused (`Auto`-gate defect) + fixed;
  index-level A/B confirms `Auto` now engages the batch path.
- **Slice 4 SPIRE×pq_fastscan** — permanent exclusion via the existing
  parse+defer behavior (an earlier parse-rejection attempt was reverted
  after reviewer 2026-06-13-01 P1).
- **Slice 2 HNSW×grouped-PQ** — left OPEN, pending the flush-width
  histogram (not closed per AC1).
- **DiskANN non-1536 TQ-QJL** — reasoned architectural boundary.

## Artifacts

- `artifacts/m5-multibit-rabitq-bench.md` — kernel microbench sweep
  (5 dims × bits 2/4), release build, IVF rabitq pg smoke.
- `artifacts/m5-index-level-bench.md` — index-level bench on real DBpedia
  10k (PG 18): IVF rabitq bit sweep with counter engagement, IVF Auto-gate
  A/B, SPIRE rabitq GUC A/B.

## Validation (M5)

- `cargo build --release`; `cargo check --all-targets --features bench`.
- `cargo test --lib` touched modules green; clippy clean.
- `cargo pgrx test pg18`: IVF rabitq build/scan/insert/vacuum + IVF recall
  smoke passed. SPIRE pq_fastscan stale tests restored to green by the
  slice-4 revert.

## Reviewer feedback addressed

`feedback/2026-06-13-01-reviewer.md`: P1 (stale pq_fastscan tests) — slice-4
reverted; P1 (HNSW grouped-PQ closure) — re-marked OPEN; P2 (missing
request.md) — this file.
