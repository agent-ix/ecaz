# Review Request: A4 Full vs Tiled FWHT — TurboQuantDB Comparison

Basis: `main` working tree, with reference to `~/dev_bak/TurboQuantDB/`

## Summary

- this packet follows `202-a4-1536-tail-truncation` and `203-a4-1536-tiled-fwht-quantizer`
- its purpose is to compare tqvector's current tiled-FWHT quantizer path against the upstream
  TurboQuantDB implementation and evaluate whether the next quality step is to adopt TurboQuantDB's
  full-FWHT + all-n-dims architecture
- three critical architectural divergences between tqvector and TurboQuantDB are documented, along
  with their quality and cost implications

## Reference Hypotheses

Continues the existing A4 labels:

1. `H1: graph-runtime gap` — secondary, verified in reviews 200/201
2. `H2: quantized-objective mismatch` — dominant, narrowed below
3. `H3: quantized-path implementation defect` — partially addressed by tiled FWHT in review 203

This packet refines H2 and H3 by comparing against the upstream reference:

- `H2a: tiled FWHT provides weaker decorrelation than full FWHT, leaving residual structure
  that scalar quantization handles poorly`
- `H3b: codebook is parameterized with dim=1536 but tiled FWHT produces per-coordinate
  marginals closer to Beta(d=512), causing a distribution mismatch`

## Three FWHT Configurations Compared

### Configuration 1: Old full FWHT + truncation (pre-review-203)

```
pad_input(vector, 2048)           → 1536 signal + 512 zeros
srht(&padded, &signs_2048)        → 2048 rotated dims (energy spread uniformly)
quantize_to_indices(..., 1536)    → keep first 1536, discard [1536, 2048)
```

- **Energy loss**: 25% of rotated signal discarded
- **Codebook**: `lloyd_max(bits-1, 1536)` → Beta(768, 768) prior
- **Actual marginal**: post-rotation in 2048 space → closer to Beta(1024, 1024)
- **Exact-only recall**: ~49.5% uniform 1k, ~55.5% clustered 1k (review 202)
- **Verdict**: worst configuration. Pays full 2048 FWHT cost, keeps only 75% of signal,
  codebook mismatched

### Configuration 2: Current tiled FWHT (post-review-203)

```
srht_tiled(&vector, &signs_1536, tile_size=512)  → 3 × 512 independent FWHTs
quantize_to_indices(..., 1536)                     → all 1536 dims retained
```

- **Energy loss**: 0% — no padding, no truncation
- **Codebook**: `lloyd_max(bits-1, 1536)` → Beta(768, 768) prior
- **Actual marginal**: 512-dim tiles → closer to Beta(256, 256)
- **Codebook mismatch**: Beta(768) vs Beta(256) — the codebook is too narrow for the actual
  per-coordinate distribution. Tiled rotation in 512-dim space produces wider marginals than
  1536-dim full rotation. The codebook underestimates tail probability.
- **Exact-only recall**: ~77-81.5% on 1k corpora (review 203)
- **Verdict**: large quality recovery from eliminating truncation. Codebook mismatch is a known
  remaining issue.

### Configuration 3: TurboQuantDB full FWHT + all-n-dims

```
pad_input(vector, 2048)           → 1536 signal + 512 zeros
srht(&padded, &signs_2048)        → 2048 rotated dims
quantize_to_indices(..., 2048)    → ALL 2048 dims encoded and stored
```

- **Energy loss**: 0% — all rotated dimensions retained
- **Codebook**: `lloyd_max(bits-1, 2048)` → Beta(1024, 1024) prior
- **Actual marginal**: full 2048-dim rotation → exactly Beta(1024, 1024)
- **Codebook mismatch**: none — codebook matches the actual post-rotation distribution
- **Encoded dimensions**: 2048 (33% more than tqvector's 1536)
- **Storage cost**: 33% larger MSE code, 33% larger QJL code
- **FWHT cost**: same as configuration 1 (both pad to 2048)
- **Verdict**: theoretically optimal for the SRHT+scalar-quantization family. Full decorrelation,
  exact codebook match, no information loss. The cost is storage.

## Detailed TurboQuantDB Divergences

### 1. Transform and encoding dimension

| Property | tqvector (current) | TurboQuantDB |
|---|---|---|
| Input dim | 1536 | 1536 |
| Transform dim | 1536 (tiled 3×512) | 2048 (padded) |
| Encoded dims | 1536 | 2048 |
| FWHT type | tiled orthonormal | full orthonormal + AVX2 |
| Padding | none | 512 zeros |

**Impact**: TurboQuantDB encodes 33% more dimensions. Every rotated coordinate carries signal
(the SRHT spreads energy uniformly), so those extra 512 dims contribute to scoring fidelity.

### 2. Codebook parameterization

| Property | tqvector (current) | TurboQuantDB |
|---|---|---|
| `lloyd_max` dim arg | `dim` = 1536 | `n` = 2048 |
| Beta prior | Beta(768, 768) | Beta(1024, 1024) |
| Matches actual distribution? | No (actual ≈ Beta(256, 256) from tiles) | Yes |

**Impact**: tqvector's codebook is generated for a distribution narrower than what the tiled
FWHT actually produces. The Lloyd-Max centroids are suboptimally placed — too concentrated near
zero, not enough coverage in the tails. This wastes bits on coordinates that rarely appear
near the codebook boundaries.

**Fix regardless of FWHT strategy**: if keeping tiled FWHT, the codebook should use `tile_dim`
(512), not `dim` (1536). This is a one-line change in `prod.rs:52`.

### 3. Centroid lookup strategy

| Property | tqvector | TurboQuantDB |
|---|---|---|
| Method | linear scan | binary search (`partition_point`) |
| Tie-breaking | higher index | lower index |
| Complexity | O(k) | O(log k) |

**Impact**: at 4-bit (8 centroids), the performance difference is negligible. At higher
bit-widths the binary search matters. Tie-breaking difference is a minor source of divergence
but not a quality driver.

### 4. Minor divergences

- **RNG**: tqvector uses ChaCha8Rng; TurboQuantDB uses StdRng. Different sign vectors for
  same seed, but both are valid PRNGs.
- **QJL scope**: TurboQuantDB's QjlQuantizer is 412 lines with stateful projection matrices;
  tqvector's qjl.rs is 14 lines. The QJL contribution is secondary (review 201: ~1% recall),
  so this gap is low priority.

## Quality Gap Analysis

| Configuration | Exact-only Recall@10 (clustered 1k) | Energy retained | Codebook match |
|---|---|---|---|
| Old full+truncate | ~55.5% | 75% | No (Beta(768) vs Beta(1024)) |
| Current tiled | ~81.5% | 100% | No (Beta(768) vs Beta(256)) |
| TurboQuantDB full+all-n | not yet measured | 100% | Yes |

The gap from old→tiled was ~26pp, driven almost entirely by eliminating truncation loss.

The remaining gap from tiled to full+all-n has two components:
1. **Decorrelation scope**: tiled FWHT decorrelates within 512-dim blocks. Full FWHT
   decorrelates across all 2048 dims. Cross-tile correlations survive tiling.
2. **Codebook accuracy**: fixable independently — change `lloyd_max` dim arg to match the
   actual post-rotation distribution.

## Cost of Adopting Full FWHT + All-n-Dims

### Storage

At 4-bit quantization:

| Metric | tqvector (1536 tiled) | Full+all-n (2048) | Delta |
|---|---|---|---|
| MSE code | 576 B | 768 B | +33% |
| QJL code | 192 B | 256 B | +33% |
| Payload | 772 B | 1,028 B | +33% |
| Vectors/page (8 KB) | ~9 | ~7 | -22% |
| 1M vectors index | ~811 MB | ~1,080 MB | +269 MB |

### FWHT cost

| Metric | tqvector (1536 tiled) | Full+all-n (2048) |
|---|---|---|
| Butterfly ops | 3 × 512 × log₂(512) = 13,824 | 2048 × log₂(2048) = 22,528 |
| Signs memory | 6 KB (1536 × f32) | 8 KB (2048 × f32) |

Tiled FWHT is ~39% cheaper in butterfly ops. But FWHT is a one-time cost per encode and per
query preparation, not per candidate — so this difference is marginal in practice.

### Per-candidate scoring

Scoring iterates over `encoded_dims`. Moving from 1536 to 2048 dims increases the per-candidate
hot loop by 33%. At scale this is the dominant cost difference.

### L1D cache pressure (Graviton)

Per ADR-021: at 4-bit, the 2048-dim LUT is 64 KB = entire L1D. The 1536-dim LUT is 48 KB.
The 2048@3bit configuration (32 KB LUT) is the recommended Graviton workaround.

## Recommended Next Steps

### Immediate (cheap, high-signal)

1. **Fix codebook parameterization for tiled FWHT** — change `prod.rs:52` from
   `lloyd_max((bits-1), dim, ...)` to `lloyd_max((bits-1), tile_dim.unwrap_or(transform_dim), ...)`.
   This fixes the Beta(768) vs Beta(256) mismatch at zero storage cost.

2. **Measure recall after codebook fix** — rerun the exact-only 1k harness from review 203.
   This isolates how much of the remaining gap is codebook mismatch vs decorrelation scope.

### Medium-term (requires implementation)

3. **Prototype full FWHT + all-n encoding** — implement the TurboQuantDB configuration as an
   opt-in mode. Encode and score all 2048 dims. Measure recall on the same harness.

4. **Compare tiled-with-fixed-codebook vs full+all-n** — if the codebook fix closes most of
   the gap, the 33% storage cost of full+all-n may not be justified. If the gap persists, the
   decorrelation improvement justifies the cost.

### Decision framework

```
codebook_fix_recall = measure(tiled + correct codebook)
full_n_recall       = measure(full FWHT + all 2048 dims)

if (full_n_recall - codebook_fix_recall) < 2pp:
    keep tiled FWHT (cheaper storage, sufficient quality)
else:
    adopt full+all-n, possibly at 3-bit to match payload size (ADR-021)
```

## Review Focus

- whether the three-configuration comparison fairly captures the architectural options
- whether the codebook fix should be attempted before prototyping full+all-n
- whether the 33% storage cost of full+all-n is acceptable given the quality ceiling it enables
- whether ADR-007's decision gates (§ Decision Gates) have been triggered by reviews 202/203,
  warranting a new ADR on the FWHT transform strategy
