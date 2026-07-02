# Review request: Task 136 — int8_approx rank-1 in-register scorer for IVF

- Task: `plan/tasks/136-tq-rank1-inregister-scorer.md`
- Branch: `task-136-rank1-scorer` (base `d13ddbe86`, the frozen
  `task-125-tq-scorer-optimization` tip)
- Code commit: `9514c7518`
- Evidence: `artifacts/manifest.md` (head SHA, commands, key result lines)

## What changed

`ec_ivf.turboquant_scorer` session GUC (`lut` default / `int8_approx`) selects
the prepared-query representation for the TurboQuant no-QJL 4-bit lane. The
`int8_approx` setting routes the IVF approximate scan through the existing
factored rank-1 kernel (`quant::int8_approx32`, Task 98, previously HNSW-only)
via a new `IvfPreparedQuery::TurboQuantNoQjl4BitInt8Approx` variant handled at
every dispatch surface in `src/am/ec_ivf/quantizer.rs`: single-candidate,
min-bound (falls through to full scoring — the factored query has no
suffix-max table), slab batch, borrowed-ref batch, and the common `QuantCodec`
batch route. Query-side only; no on-disk format/mode/reloption change (hard
constraint respected). Default is unchanged (`lut`).

Validation: 38 focused `am::ec_ivf::quantizer` tests pass (5 new int8_approx
dispatch/parity tests, bit-equal batch-vs-scalar assertions); clippy pg18 gate
carries only the two documented pre-existing findings. pgrx runtime tests
skipped per the macOS `_BufferBlocks` policy; behavior validated end-to-end by
the bench suite below.

## A/B result (10k/50k/100k, `ecaz bench suite`, same session/tables/binary)

- **Latency: −14.1% / −14.3% / −16.5% mean** (0.92→0.79, 1.89→1.62,
  2.79→2.33 ms) at 10k/50k/100k; p50 −15% / −14% / −18%.
- **Scorer stage: −33% / −32% / −35%** per-sweep (`scorer_batch`
  24.4→16.3, 39.4→26.7, 38.6→25.2 ms); non-scorer stages unchanged within
  noise, so attribution is clean.
- **Recall@10: within noise at every scale** — 0.9734→0.9719 (−0.15 pp),
  0.9521→0.9521 (±0), 0.8969→0.8938 (−0.31 pp); all deltas well inside the
  lut ci95 and far from the Task 98 fallback trigger. The i16 factored
  fallback variant was therefore not needed.
- Storage unchanged by construction (query-side only).

This meets the task gate ("a measurable IVF latency win at recall within noise
of the LUT path"). The Task 133 stage-share prediction (scorer 46% of the
approximate scan → ~2× kernel win ≈ 20%+ e2e) roughly held: the kernel came in
at ~1.5× and e2e at 14–17%.

## Asks

1. Review the dispatch wiring in `quantizer.rs` (match-arm coverage, no new
   `unsafe fn`, anti-pattern B clean) and the GUC surface in `options.rs`.
2. Decide default: evidence supports flipping `ec_ivf.turboquant_scorer` to
   `int8_approx` (recall within noise at all three scales, latency win at all
   three). I left the default at `lut` so the flip is a reviewer/owner call;
   if approved I'll land the default flip + docs as a follow-up slice with a
   confirming A/B cell.
3. Known follow-ups (not blockers): Graviton/AWS lane evidence remains the
   standing open item for the task-125 family; 1m scale encouraged by policy
   if the default flips.
