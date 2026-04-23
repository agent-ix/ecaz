# Review Request: Task 25 Slice 5 — RaBitQ Feasibility Binary (Phase 2 Harness)

Scope:
- `src/bin/rabitq_feasibility.rs` — new. Loads a corpus + query set
  (TSV via `--corpus-file` / `--queries-file`, or synthetic
  Gaussian as a default), encodes the corpus under the slice-4
  `RaBitQQuantizer::with_srht`, computes exact top-K by brute-force
  inner product, runs the slice-4 asymmetric estimator against
  every encoded vector, measures recall@10, and reports:
  - storage ratio vs. PQ4 (`RaBitQ 200 B` vs. `PQ4 768 B` at D=1536)
  - mean / p50 / p99 of the Cauchy-Schwarz error bound across the
    recalled top-K
  - mean / p50 / p99 of the realized `|estimate − truth|` error
  - `tightness = mean(error) / mean(bound)` — the calibration
    number Stage 3 (task 27) cares about
  - a PASS / MARGINAL / FAIL verdict against the ADR-045 Stage 1
    gate (`recall@10 within 1 pp = PASS`, `1–2 pp = MARGINAL`,
    `>2 pp = FAIL`)
- `src/lib.rs` `bench_api` — exports `RaBitQQuantizer`,
  `SrhtRotation`, `Rotation`, `PreparedEstimator`, `DistanceEstimate`,
  the `Quantizer` / `QueryScorer` traits, and the `RABITQ_*_LEN`
  constants. Binaries only see the library crate's public surface,
  so this is what gets re-exported for the feasibility binary and
  for future consumers (task 27 offline tooling).

Task: `plan/tasks/25-rabitq-quantizer.md` (Phase 2, slice 5 of 6).

Branch: `task25-rabitq-stage1-phase0` (slice 5 builds on `5bcf78a`).

## Problem

Phase 2 is where the ADR-045 Stage 1 gate is actually decided.
Slices 1–4 built the quantizer and the estimator; slice 5 lands the
harness that runs the estimator against real data and either clears
the gate or declares the null result. The harness needs to be
shippable as a standalone binary (so it can run on whichever
machine holds the canonical 50k / 1M corpus seams), produce output
that is grep-able from a review packet, and avoid a hard
dependency on pgrx server state (the `cargo run --bin` target
compiles against the library, which pulls `pgrx`, but the binary
itself does not call into the server runtime).

## Approach

### Brute-force truth, not Postgres-in-the-loop

The feasibility binary computes exact top-K via a plain
`Σ q_i · c_i` loop on the in-process corpus rather than spinning up
a Postgres instance with `ec_hnsw` and a competing index. Three
reasons:

1. Phase 2's question is "does the estimator have the recall budget
   *at all*." That question is answered by comparing the estimator
   to exact scoring, not to another ANN index.
2. Standing up a pgrx extension inside a `bin` target is awkward
   and slow; keeping the binary library-only means it runs on
   laptops and CI without a Postgres cluster.
3. The 50k / 1M seams used for recall already live as TSVs (task
   10054 / 12). `--corpus-file path.tsv` reads them directly.

### Synthetic default is a smoke test, not a gate input

When `--corpus-file` / `--queries-file` are absent, the binary
synthesizes deterministic Gaussian vectors from `rand_chacha`. A
local run with `--corpus 500 --queries 50 --dim 256 --top-k 10`
printed `recall@10 mean: 0.43` → `GATE: FAIL`. That is expected
behavior for binary quantization on iid Gaussians at D=256 (no
rotation benefit, no corpus structure); the result does **not**
inform the ADR-045 gate, it only proves the binary runs
end-to-end and the estimator/bound arithmetic is live.

The gate decision requires the real seams:

```
./target/release/rabitq_feasibility \
    --corpus-file data/glove-1m.tsv \
    --queries-file data/glove-1m-queries.tsv \
    --dim 1536 --top-k 10
```

That run must happen on the machine with the TSV seams; its
output gets filed as the review packet for the gate decision
(a follow-up packet, not this one — slice 5 ships the binary,
the packet reporting the gate verdict is paired with slice 6's
handoff contract).

### Output shape

Output is plain-text key/value lines designed to be copy-pasted
into the gate packet without transformation:

```
recall@10 mean: 0.9843
bound  mean=3.412  p50=3.180  p99=6.927
error  mean=0.142  p50=0.118  p99=0.548
tightness (error / bound) mean: 0.041
GATE: PASS (recall gap 0.157 pp ≤ 1.0 pp)
```

`tightness` is the number Stage 3's candidate-pool sizer wants:
smaller means the bound envelopes the true error loosely (which is
actually OK for correctness; tighter bounds let Stage 3 pick
smaller pools at the same safety margin).

### Storage parity check is first-class output

`# storage: RaBitQ code N B, PQ4 code M B (parity ratio ...)`
prints before any measurement. The ADR-045 gate is defined *at
PQ4-parity storage*, so the binary makes the parity ratio visible
up front — if a future layout change made RaBitQ cheaper or more
expensive than expected, the ratio line would catch it before the
reader reads the recall number.

## Verification

- `cargo build --release --bin rabitq_feasibility` clean.
- Smoke run (`--corpus 500 --queries 50 --dim 256 --top-k 10`)
  completes in <1 s and exercises the full pipeline: corpus
  synthesis → encode → per-query exact + estimator scoring →
  summary stats → gate verdict print. The FAIL result on synthetic
  D=256 data is the correct behavior for that input and is **not**
  a gate signal.

## What this slice does NOT do

- No actual gate-decision run on the 50k / 1M seams. That is
  environment work that I cannot run from this session (no access
  to the canonical TSV files). The gate packet is a follow-up to
  slice 6; once someone runs the binary against the real seams,
  the output drops into a new `review/20005-*` packet along with
  the PASS / MARGINAL / FAIL decision. Slice 6 below writes the
  handoff contract assuming PASS (the contract freezes the API
  surface either way; only task 27's start date depends on the
  verdict).
- No SIMD in the estimator inner loop (flagged in slice 4). If the
  feasibility run on the real seams is slow enough to matter for
  sweep iteration, SIMD gets its own slice after the gate clears.

## Open questions for reviewer

1. Is the library-only harness shape right, or would you rather
   the binary wrap a `pgrx` server and measure recall through a
   real `ec_hnsw` scan with RaBitQ as a substitute scorer? I took
   the library path per the task doc ("same shape as
   `aq_feasibility`"); the pgrx-in-the-loop version would measure
   something different (the index's orchestration overhead) and is
   more naturally task 27 territory.
2. The synthetic default's RNG is `rand_chacha`; the same crate is
   already a transitive dep, so I pulled it in directly. The
   slice-4 estimator tests used a hand-rolled Box-Muller to avoid
   the test dep. Happy to unify on `rand_chacha` across both if
   you'd rather not carry two RNG paths in the module.
3. Should the verdict print also include a per-bit-budget sweep
   (the task mentions "bit-budget sweep")? The slice-4 layout is
   fixed at `D/8 + 8 B`, so "sweep" would mean running at several
   `RABITQ_*_LEN` configurations. That is a real study but means
   changing the encoder's layout at runtime — I'd rather do it as
   a second binary (`rabitq_bit_sweep`) or a `--layout` flag in a
   follow-up if the recall gate clears at the default layout.
