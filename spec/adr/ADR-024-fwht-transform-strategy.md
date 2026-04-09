---
id: ADR-024
title: "FWHT Transform Strategy: Full vs Tiled for Non-Power-of-2 Dimensions"
status: PROPOSED
impact: Affects FR-013, NFR-003, ADR-006, ADR-007, ADR-020, ADR-021
date: 2026-04-08
---
# ADR-024: FWHT Transform Strategy — Full vs Tiled for Non-Power-of-2 Dimensions

## Context

The FWHT requires power-of-2 input lengths. For non-power-of-2 dimensions (most importantly
1536), tqvector must choose a transform strategy. Three configurations have been evaluated
across reviews 200–204 and compared against the upstream TurboQuantDB implementation.

### ADR-007 Decision Gates Have Been Triggered

ADR-007 §3 chose to "persist only the first `d` transform coordinates" and defined explicit
decision gates (ADR-007, lines 163–174):

> The current truncated-tail design remains the default if [...] its Recall@10 degradation
> versus the tail-retaining reference variant is no more than 1.5 percentage points

Reviews 202 and 203 measured the degradation at **~26 percentage points** (55.5% → 81.5% on
clustered 1k). The gate failed by an order of magnitude. ADR-007's follow-up clause (line 97)
explicitly anticipated this:

> If future benchmarks show unacceptable loss from transform-tail truncation, a later ADR may
> introduce a full-n storage mode or an alternate packed layout.

This ADR is that later ADR.

### Three Evaluated Configurations

| Configuration | Transform | Encoded dims | Energy retained | Codebook match | Recall@10 (clustered 1k) |
|---|---|---|---|---|---|
| **A: Full + truncate** (ADR-007 original) | Full FWHT on 2048 | 1536 | 75% | No | ~55.5% |
| **B: Tiled** (current, review 203) | 3×512 tiled FWHT | 1536 | 100% | No | ~81.5% |
| **C: Full + all-n** (TurboQuantDB) | Full FWHT on 2048 | 2048 | 100% | Yes | not yet measured |

Configuration A is retired. The decision is between B and C.

### What TurboQuantDB Does Differently

The upstream TurboQuantDB implementation (`~/dev_bak/TurboQuantDB/`) uses configuration C:

1. **Pad to `n = next_power_of_two(d)`** — for d=1536, n=2048
2. **Full FWHT on n dimensions** — complete decorrelation across all coordinates
3. **Encode and store all n dimensions** — no truncation
4. **Codebook uses n, not d** — `lloyd_max(bits-1, n)` → Beta(1024, 1024), exactly matching
   the post-rotation marginal distribution
5. **Score over all n dimensions** — LUT and hot loop iterate over 2048

tqvector's current tiled configuration (B) has two remaining divergences from TurboQuantDB:

1. **Decorrelation scope**: tiled FWHT decorrelates within 512-dim blocks, not globally.
   Cross-tile correlations survive. Full FWHT eliminates all correlations.
2. **Codebook mismatch**: `lloyd_max` receives `dim=1536` but the actual per-coordinate
   marginal from 512-dim tiled rotation is closer to Beta(256, 256). The codebook centroids
   are too concentrated near zero.

## Decision

### 1. Fix codebook parameterization immediately (zero-cost)

Change `prod.rs:52` from:

```rust
let codebook = codebook::lloyd_max((bits - 1) as usize, dim, 20_000);
```

to use the effective rotation dimension:

```rust
let cb_dim = rotation::tile_dim(dim).unwrap_or(rotation::transform_dim(dim));
let codebook = codebook::lloyd_max((bits - 1) as usize, cb_dim, 20_000);
```

This eliminates the Beta(768) vs Beta(256) mismatch at zero storage cost. It changes encoded
values for 1536-dim vectors (acceptable pre-release per ADR-021 §Wire format compatibility).

### 2. Measure the codebook fix before committing to full+all-n

Rerun the exact-only 1k harness from review 203 with the corrected codebook. This isolates
how much of the remaining gap to TurboQuantDB is codebook mismatch (fixable for free) vs
decorrelation scope (requires full FWHT + storage increase).

### 3. Adopt full FWHT + all-n-dims only if the decorrelation gap exceeds 2pp

If the codebook fix closes the gap to within 2 percentage points of full+all-n, keep tiled
FWHT. The 33% storage savings justify a small quality tradeoff.

If the gap persists above 2pp, adopt full FWHT + all-n-dims as the primary path for
non-power-of-2 dimensions. In that case, the recommended production configuration is
2048@3bit per ADR-021, which has identical payload size to 1536@4bit (772 bytes).

### 4. Keep tiled FWHT as a supported mode

Even if full+all-n becomes the default, tiled FWHT remains valuable for:
- Storage-constrained deployments where 33% savings matter
- Future higher dimensions (3072, 4096) where full FWHT may be too expensive
- Dimensions that factor cleanly into power-of-2 tiles

### 5. Retire configuration A (full + truncate)

The original ADR-007 §3 truncation strategy is permanently retired. It is dominated by both
tiled FWHT (same storage, +26pp recall) and full+all-n (more storage, even better recall).
No code path should produce configuration A going forward.

## Consequences

### Positive

- The codebook fix (decision 1) is a zero-cost quality improvement that can ship immediately
- The measurement-first approach (decision 2) avoids committing to 33% more storage without
  evidence that it's needed
- Tiled FWHT remains available as a lighter-weight option (decision 4)
- The decision framework is empirical, not theoretical — the specific recall threshold (2pp)
  is testable on the existing harness

### Negative

- If full+all-n is adopted, 1536-dim vectors pay 33% more storage (772 B → 1,028 B per vector)
- The per-candidate scoring hot loop grows by 33% (1536 → 2048 iterations)
- L1D cache pressure increases (48 KB → 64 KB LUT at 4-bit), borderline on Graviton
- Tiled FWHT code must be maintained even if full+all-n is the default

### Neutral

- For power-of-2 dimensions (1024, 2048), nothing changes — full FWHT with no padding is
  already optimal
- The codebook fix changes encoded values, but tqvector has no stable release — no migration
  needed
- The 2048@3bit workaround (ADR-021) neutralizes the storage cost if 3-bit recall is adequate

## Open Questions

1. **Codebook fix recall delta**: How much recall does the codebook fix alone recover? This
   is the single most important measurement for this ADR.
2. **Full+all-n recall at 1536**: What is the exact-only recall when encoding all 2048 dims
   with correct codebook? This establishes the ceiling for configuration C.
3. **Structured data sensitivity**: Do the relative rankings of B and C change on real
   embedding data vs synthetic corpora? Review 201 recommended structured-data testing.
4. **3-bit viability**: Does 2048@3bit match or exceed 1536@4bit recall? ADR-021 hypothesizes
   yes but has no measurement.

## References

- ADR-006: Own quantizer — extraction from TurboQuantDB
- ADR-007: Persist gamma and raw-query scoring — §3 truncation decision and §Decision Gates
- ADR-020: Embedding dimension operating points
- ADR-021: Default vector dimension 2048 — tiled FWHT and 2048@3bit analysis
- Review 200: A4 recall gate rerun — graph search near-optimal relative to ceiling
- Review 201: A4 quantizer triage — scorer ablation, structured data recommendations
- Review 202: A4 1536 tail-truncation probes — transform-tail truncation identified as dominant loss
- Review 203: A4 1536 tiled-FWHT quantizer — production tiled FWHT implementation
- Review 204: A4 full vs tiled FWHT — TurboQuantDB comparison and three-configuration analysis
- TurboQuantDB source: `~/dev_bak/TurboQuantDB/src/quantizer/prod.rs`, `mse.rs`, `../linalg/hadamard.rs`
