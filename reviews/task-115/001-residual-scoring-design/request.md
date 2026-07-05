# Task 115 / 001 — Phase 1: residual scoring design + scalar reference tests

Branch: `task-115-ivf-rabitq-residual-quantization`
Code commit: `0225febd2` (Phase 1 scoring + scalar reference tests)
Phase: 1 (Scoring Design). Phases 2–3 (gated build/insert + scan integration)
follow in later slices; Phases 4–5 (recall-per-probe + latency) are env-blocked
and deferred to the bench host with ready-to-run configs.

## The residual scoring formula

For a posting vector `o` assigned to its IVF centroid `c`, with query `q`:

```text
⟨q, o⟩ = ⟨q, c⟩ + ⟨q, o − c⟩
```

- `⟨q, c⟩` is computed **exactly** once per probed list. The centroid is already
  loaded for list selection, so this is amortized per-list with zero per-posting
  cost.
- `⟨q, o − c⟩` is estimated by the **unchanged** RaBitQ asymmetric estimator run
  on a code that encodes the residual `r = o − c`.

The key simplification: because `prepare_estimator(q)` rotates `q` and the
residual code packs `sign(rotate(o − c)) = sign(rotate(o) − rotate(c))` (rotation
is linear), the existing estimator's output on a residual code *is* `≈ ⟨q, o − c⟩`.
**No query-side change, no new estimator, no new kernel** — residual mode reuses
the entire plain RaBitQ scoring stack and only changes the vector that is encoded
at build/insert time.

## Correction metadata + the Phase-1 stop condition

The residual code is byte-for-byte the **same shape** as the plain code:
`⌈D·bits/8⌉` packed bytes + the 12-byte scalar tail (`||r||`, `r_dot`,
`||x_dec||`). The scalars now describe the residual `r = o − c` instead of `o`,
but there is **zero extra per-posting metadata**. The centroid term is exact and
per-list, not stored per-posting.

**The Phase-1 stop condition does NOT fire.** Residual mode has identical index
size to plain RaBitQ. This is the ideal outcome for the compact-index goal: same
bytes, smaller residual dynamic range → expected lower quantization error → the
recall-per-probe win the task is chasing (to be confirmed on the bench host,
Phase 4).

This is the same insight as the existing Symphony "centered" path
(`encode_code_centered`), specialized to a fixed per-list center instead of a
per-vertex one.

## What landed (commit `0225febd2`)

In `src/quant/rabitq.rs`:
- **`RaBitQQuantizer::encode_code_residual(v, centroid)`** — encodes `v − centroid`
  via the identical `encode_code` body. Asserts dim match; returns the same-length
  code.
- **`RaBitQQuantizer::combine_residual_estimate(centroid_ip, residual_estimate)`**
  — the one-line `+` that documents the scoring identity in one place, shared by
  the AM scan (later slice) and the reference tests.

## Tested green (scalar reference, `cargo test`, SRHT rotation, bits 1/2/4/8)

See `artifacts/phase1-residual-tests.log` (4 passed):
- `residual_code_is_byte_identical_shape_to_absolute_code` — residual code length
  == plain code length (the stop-condition evidence).
- `residual_estimate_recovers_exact_residual_term` — `encode_code_residual(o,c)`
  is bit-for-bit equal to `encode_code(o − c)`, and the estimate matches.
- `residual_scoring_matches_exact_within_tolerance` — combined estimate matches
  exact `⟨q, o⟩` within quantization tolerance (error measured relative to the
  residual-term scale `||q||·||o − c||`, since the full sum can be ~0 for a random
  query/centroid even when absolute error is tiny).
- `residual_beats_absolute_on_concentrated_lists` — on a list of centroid +
  small jitter, residual mean abs error < absolute mean abs error at bits=1.

## Coordination with Task 113 (preview; handled in the scan slice)

113's posting-prune cutoff `||o||·||q||/|o_dot|` upper-bounds the estimate of
`⟨q, o⟩` for **plain** payloads. Under residual encoding the quantized estimate is
`⟨q, o − c⟩`, so the sound full-score upper bound becomes
`⟨q, c⟩ + ||r||·||q||/|r_dot|`, i.e. the per-list cutoff must be shifted by
`−⟨q, c⟩`. The current scan scoring sites do not carry the centroid term, so the
recall-safe plan for the scan slice is to **gate the 113 posting-prune OFF when
residual mode is active** (conservative, byte-identical to unpruned), with the
shifted-cutoff as the documented follow-up lever. A residual-mode pruned==unpruned
recall-safety test will ship with that slice.

## Validation notes

- `cargo clippy --lib --no-default-features --features pg18 -- -D warnings`: clean.
- No pg18 behavior touched in this slice (pure quantizer math + tests), so no
  `cargo pgrx test` was needed here; the gated build/insert/scan slices will carry
  the pgrx coexistence + recall-safety tests.

## Artifacts

- `artifacts/manifest.md`
- `artifacts/phase1-residual-tests.log`
