# Review Request: Task 25 Slice 12 — Q-bit RaBitQ Encoder + Recall Sweep

Scope:
- `src/quant/rabitq.rs`:
  - New `RABITQ_SUPPORTED_BITS = [1, 2, 4, 8]` public constant.
  - `RaBitQQuantizer::with_bits(rotation, bits)` /
    `::with_srht_bits(dim, prod, bits)` constructors. Returns
    `Err(String)` for unsupported bit counts.
  - `bits_per_dim: u8` field + `bits_per_dim()` / `packed_bytes()`
    accessors.
  - `encode_code` generalized: at `q = 1` packs sign bits (canonical
    RaBitQ); at `q ∈ {2, 4, 8}` each rotated-unit coordinate is
    scaled by `√D`, clipped to `±2σ`, binned into `2^q` uniform
    signed levels, and packed LSB-first.
  - `estimate_ip_impl` generalized. New scalar layout: `||o||`
    (4 B), `o_dot` (4 B), `||x_dec||` (4 B). Adding the stored
    `||x_dec||` lets one formula handle all `q` — at `q = 1` the
    norm is always `√D` but we store it for layout uniformity.
  - Estimator formula (paper's asymmetric form, derivation in the
    code comment):
    ```
    ⟨q, o⟩ ≈ ||o|| · Σ q_i · dequant(level_i) / (o_dot · ||x_dec||)
    ```
    At `q = 1` this reduces bit-identical to the slice-9 form.
  - Four new unit tests: `qbit_code_len_scales_with_bits`,
    `qbit_encoder_reduces_error_vs_binary` (q=4 strictly lower
    mean estimator error than q=1 on five Gaussian seeds),
    `qbit_rejects_unsupported_bits`, plus the retained slice-9
    tests unchanged.
- `src/lib.rs` `bench_api` — adds `RABITQ_SUPPORTED_BITS`,
  `RABITQ_XNORM_LEN`.
- `crates/ecaz-cli/src/commands/quant/feasibility.rs`:
  - `--bits {1, 2, 4, 8}` flag on `ecaz quant feasibility`,
    default `1`. Header prints `bits/dim=N` alongside the storage
    ratio.

Task: `plan/tasks/25-rabitq-quantizer.md` (slice 12 of the
extended plan; addresses the task-doc ambiguity between "1 bit/dim"
in the encoding section and "match PQ4 storage" in the gate).

Branch: `task25-rabitq-stage1-phase0` (slice 12 builds on `a707395`).

Artifact: `artifacts/sweep-dbpedia-10k.txt` — verbatim output of a q ∈ {1, 2,
4, 8} sweep on DBpedia-10k.

## Results (DBpedia-10k, no rerank, paper-faithful estimator)

| bits/dim | code size | vs PQ4  | recall@10 | gap      | mean err | mean bound | verdict   |
|----------|-----------|---------|-----------|----------|----------|------------|-----------|
|    1     | 204 B     | 0.27×   | 0.8975    | 10.25 pp | 0.010    | 0.050      | FAIL      |
|    2     | 396 B     | 0.52×   | 0.9430    |  5.70 pp | 0.005    | 0.024      | FAIL      |
|    4     | 780 B     | **1.02×** | 0.9790  | **2.10 pp** | 0.002 | 0.010      | FAIL (−0.1 pp from MARGINAL) |
|    8     | 1548 B    | 2.02×   | **0.9865** | **1.35 pp** | 0.002 | 0.008    | **MARGINAL** |

`tightness = error / bound` stays in 0.21–0.24 across all `q` —
the ε-concentration bound calibration from slice 9 is preserved.

### Verdict shift vs. slices 8–10

Slices 8 and 10 both landed FAIL at 10 pp on DBpedia-10k at `q = 1`.
Slice 12 preserves the `q = 1` FAIL (which is what Symphony Stage-3
specifically cares about) but **shows that the module reaches
MARGINAL at `q = 8`**, and comes within 0.1 pp of MARGINAL at
PQ4-parity storage (`q = 4`). Per the task rubric:

> **Marginal** (within 1–2 pp) → keep the module as a non-
> rerank-eliminating quantizer under ADR-032; do not commit to
> Stages 2–3 yet; return to OPQ (task 20) to close the gap via a
> learned rotation.

So the actionable outcome is:
- **Symphony Stage 3 (no-rerank at 1 bit/dim)** still does not
  clear the gate. Task 27's start stays deferred.
- **But** the module is now demonstrably a tunable quantizer with
  a clear recall/storage tradeoff — not a dead-end experiment.
  Users who have PQ4-parity storage budget can get 0.98+ recall
  today, and the MARGINAL zone is within reach of OPQ (task 20)
  or any tighter rotation.

## Design choices

### Why `bits ∈ {1, 2, 4, 8}` only

These are byte-aligned within each coordinate, so bit packing
reduces to a small-integer shift. Supporting `q = 3, 5, 6, 7`
requires per-coordinate bit-level offset arithmetic — doable but
adds roughly 30% code volume to `write_level` / `read_level`
without a clear use case. Easy follow-up slice when someone
wants `q = 3` specifically.

### Why store `||x_dec||` when it is constant at `q = 1`

Keeping a single scalar layout across all `q` collapses the
encoder and estimator bodies to one code path. The 4 B cost at
`q = 1` (code 200 → 204 B) is negligible against the PQ4
reference (768 B).

### Scalar quantization choice: uniform on `±2σ`

Each rotated unit-vector coordinate has std ≈ `1/√D`; scaling by
`√D` puts the distribution on `N(0, 1)`. Clipping at `2σ` covers
~95% of mass and keeps quantization levels populated; past `3σ`
the outer bins become starved and waste bits. The paper's
extended RaBitQ uses a Lloyd-Max quantizer tuned for Gaussian;
that is strictly better (lower expected quantization error) but
requires per-dim calibration. Uniform-on-`±2σ` is the simple
path that still shows the expected recall scaling.

If the MARGINAL verdict at `q = 8` is worth chasing, a
follow-up slice swaps uniform binning for Lloyd-Max at compile
time — likely to close the remaining 1.35 pp gap further.

### Bound formula is unchanged at `q > 1`

The ε-concentration bound in `estimate_ip_impl` is still the
binary-case form `ε² = (1 − o_dot²) / (D · o_dot²)`. At `q > 1`
this over-estimates error (the actual residual variance is
smaller). Stage 3's pool sizer would over-allocate but not miss;
it is safe-but-loose. A tighter q-aware bound is a follow-up.

### `qbit_encoder_reduces_error_vs_binary` as a regression

Rather than asserting absolute recall numbers (sensitive to
corpus, rotation seed, RNG), the regression asserts the inequality
`err_q4 < err_q1` across five Gaussian seeds. That's a structural
property — if q=4 ever fails to beat q=1, we have a bug. Flaky-
proof without pinning to specific error values.

## One bug caught and fixed during slice 12

The first draft of the estimator used the least-squares α form
`α = ||o||·o_dot/||x_dec||`, `estimate = α · ⟨q, x_dec⟩`. Unit
tests passed but the feasibility harness showed `tightness = 5.4`
at `q = 1` (error exceeded bound by 5×!). Root cause: the α form
preserves ranking but under-scales absolute estimates by
`1/o_dot²`, breaking the bound-vs-error relationship.

Fix: switched to the paper's form
`estimate = ||o|| · ⟨q, x_dec⟩ / (o_dot · ||x_dec||)`. At `q = 1`
this reduces bit-identical to the slice-9 formula. Tightness
dropped to 0.21 (healthy). Recall at `q = 1` moved from 0.8935
back to 0.8975 (matches slice 10 verdict, confirms behavior
preservation under the layout change).

The code comment at the fix site records this derivation so
future readers see why the "cleaner-looking" α form is wrong.

## Verification

- `cargo check --lib` clean.
- `cargo test --lib quant::rabitq` — 10 pass (7 retained + 3 new
  q-bit tests).
- `cargo build --release -p ecaz-cli` clean.
- Sweep run recorded in `artifacts/sweep-dbpedia-10k.txt`, ~80 s wall total
  for four bit settings.

## What this slice does NOT do

- No `bits`-aware bound (flagged above).
- No Lloyd-Max scalar quantizer (q > 1 uses uniform binning).
- No `q ∈ {3, 5, 6, 7}`.
- No amendment to the slice-6 handoff contract. The contract's
  type signatures are still correct; an amendment adding the
  `bits_per_dim` story is a follow-up combined with the Lloyd-Max
  / q-aware-bound work.
- No seed plumbing (slice 13).

## Open questions for reviewer

1. At PQ4-parity storage (q = 4), recall is 0.9790 — exactly at
   the FAIL/MARGINAL boundary (2.1 pp vs. 2.0 pp). Changing the
   clip to `±2.5σ` or switching to Lloyd-Max likely flips it to
   MARGINAL. Worth chasing now, or leave for a follow-up?
2. MARGINAL at q = 8 means "keep the module, run OPQ before final
   decision." OPQ is task 20 — not-yet-started. Should slice 12's
   landing re-prioritize task 20 vs. leave it at its current
   position in the queue?
3. The ε-bound's `C = 2.5` constant was picked for `q = 1`. At
   `q = 8` the tightness is 0.24 — there is headroom to lower
   `C` (tighter bound) for the same confidence level. Not a
   functional issue for this slice.
