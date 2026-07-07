# Manifest — Task 162 packet 004 (parity re-measure after feedback fixes)

- Head SHA of measured build: `fb1f67eb6` (extension code `9b4c2e96b`;
  branch `task-162-ec-distann-m0`)
- Task bucket: `reviews/task-162/004-parity-remeasure/`
- Host/DB/protocol: identical to packet 002 (Intel desktop, PG18.3 port
  28818, database `ec_distann_bench`, release install verified by the
  precheck step in `precheck-host.log`, recall-before-latency per arm,
  one index per replicated table).
- What changed since packet 002's measurement: D7 default flipped to
  rabitq + iterative deepening (`56738551d`), and the finding-3 levers —
  per-record 32-wide batched neighbor-code scoring + heap rerank
  prefetch (`9b4c2e96b`). Indexes rebuilt fresh by this run's load steps.
- Command: `./target/release/ecaz --host /home/peter/.pgrx --port 28818
  --database ec_distann_bench bench suite run --config
  reviews/task-162/002-m0-bench-cells/task-162-m0-suite.json
  --only-tag setup --only-tag diskann --only-tag distann_rbq
  --continue-on-error --artifact-dir
  reviews/task-162/004-parity-remeasure/artifacts` (2026-07-07). All
  selected steps succeeded (`suite-manifest.json`).

## Current-head validation (finding 1 closure)

- `clippy-pg18-head.log`: `cargo clippy --all-targets
  --no-default-features --features pg18 -- -D warnings` — clean.
- `pg18-ec-distann-tests-head.log`: `cargo pgrx test pg18 ec_distann` —
  **45/45 green** at head (includes the corrected GUC defaults test, the
  default-codec pin, the LIMIT>top_k deepening regression, and the
  REINDEX cache-invalidation test).

## Parity A/B (rabitq both sides; recall@10 / warm p50; results.jsonl)

10k — diskann: 0.9990@3.25ms (L64), 0.9995@3.53ms, 1.0000@3.79ms.
distann: 0.9935@1.98ms (top_k=16), 0.9990@2.60ms, 0.9995@4.10ms,
1.0000@5.81ms.
Matched-recall ratios: 0.9990 → **0.80×**, 0.9995 → **1.16×**, 1.0000 →
1.53×.

50k — diskann: 0.9700@3.82ms, 0.9860@4.36ms, 0.9905@5.07ms,
0.9950@6.72ms, 0.9965@9.66ms.
distann: 0.9150@2.38ms, 0.9545@3.11ms, 0.9840@5.07ms, 0.9880@7.23ms,
0.9950@13.6ms.
Matched-recall ratios: ~0.986 → **1.16×** (0.9840@5.07 vs 0.9860@4.36),
0.9950 → **2.03×** (13.6 vs 6.72 ms).

## Levers verdict (finding 3)

Point-for-point vs packet 002 (same protocol, fresh builds): 50k distann
p50s are 2.38/3.11/5.07/7.23/13.6 ms vs 2.41/3.12/4.99/6.93/13.7 ms —
**within noise; the batched scoring + prefetch levers are inert on the
warm 50k tail.** Recall values identical (build determinism confirmed
across rebuilds).

Where the tail time actually goes (arithmetic, not profile): at
top_k=200 the scan expands ~200 records; each expansion is one 6,612 B
record read (one record per page at R=32/dim 1536) plus one exact heap
read of a ~6 KB toasted vector — the D11 `records read == reranked`
contract. diskann at its 0.995 point reads ~800 records of 432 B
(18/page) but exact-reranks only its 64-candidate rerank budget. Per
expanded candidate distann pays ~68 µs vs diskann's ~12 µs per frontier
candidate. Cache-warm prefetch and kernel batching don't touch either
term.

## Options recorded for the parity decision (finding 3 disposition)

1. **Accept a banded M0 exit**: parity holds through the ~0.988 recall
   band (≤1.16× both scales); document the ≥0.995 gap as a known
   single-node cost of the D11 exact-per-expansion contract, revisit at
   the M4 gate where the comparison is against IVF/HNSW anchors (which
   distann's absolute numbers already beat: 13.6 ms at 0.9950 vs the
   37.6 ms IVF-100k anchor).
2. **Record packing**: two records per page needs ≤ ~4,080 B → R=16 at
   rabitq (3,412 B). Halves record-read I/O; graph quality at R=16 needs
   its own A/B.
3. **Spec-level rerank economy**: allow the expansion response to defer
   `exact_dist` for beam-tail candidates (code-ranked exact-read budget,
   diskann-style). Contradicts FR-079's exact-with-expansion contract
   and the D11 equality — an ADR-085 amendment, not a code tweak.
