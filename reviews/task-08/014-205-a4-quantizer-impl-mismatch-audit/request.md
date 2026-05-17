# Review Request: A4 Quantizer Implementation Mismatch Audit

Basis: `main` working tree after review `204`, plus upstream reference at
`~/dev_bak/TurboQuantDB/`

## Why This Packet Exists

Review `204` correctly identified that tqvector's current `1536` tiled-FWHT path still diverges
from the upstream TurboQuantDB implementation. But it also made two stronger claims that were not
yet measured:

1. tiled FWHT should switch its codebook parameterization from `dim=1536` to `tile_dim=512`
2. tqvector and TurboQuantDB differ in centroid tie-breaking in a way that may affect quality

This packet checks those claims directly on the current tree and records the failed paths so they
are not retried blindly.

## Confirmed Upstream Mismatches

These are factual after re-reading upstream code:

- TurboQuantDB rotates to `n = next_power_of_two(d)` and keeps `n` for MSE scoring:
  - `MseQuantizer::new(d, b, seed)` builds centroids with `lloyd_max(b, n, ...)`
  - `MseQuantizer::quantize` emits `n` indices
  - `ProdQuantizer::prepare_ip_query` builds `n * codebook_len` LUT entries
  - `PreparedIpQuery::qjl_scale` divides by `n`
- tqvector's current tiled path for `1536`:
  - rotates/stores/scores `1536`
  - builds the codebook with `lloyd_max(bits-1, 1536, ...)`

So the live architectural mismatch with upstream is real:

1. `full-n` vs tiled transform scope
2. `full-n` vs `d` encoded/scored dimensions
3. `n`-parameterized codebook vs `d`-parameterized codebook

## Correction to Review 204

The tie-break claim in review `204` was wrong.

- tqvector's [mse.rs](/home/peter/dev/tqvector/src/quant/mse.rs) keeps the first centroid on exact
  distance ties because it updates only on `<`, not `<=`
- TurboQuantDB's `partition_point` path also resolves exact midpoint ties to the lower index

I added a unit test to pin this down:

- `quant::mse::tests::nearest_centroid_index_prefers_lower_index_on_tie`

So centroid tie-breaking is not a meaningful divergence between the two implementations.

## New Exact-Only Experiments

New ignored probes were added in [recall_integration.rs](/home/peter/dev/tqvector/tests/recall_integration.rs):

- `quantizer_recall_1536_tiled_codebook_dim_sweep_1k_uniform`
- `quantizer_recall_1536_tiled_codebook_dim_sweep_1k_clustered`
- `quantizer_recall_1536_full_tail_exact_1k_clustered`

These tests hold the current production tiled-FWHT path constant and swap only the codebook
parameterization:

- `prod_cb512`  → `lloyd_max(bits-1, 512, ...)`
- `prod_cb1536` → `lloyd_max(bits-1, 1536, ...)`   current production behavior
- `prod_cb2048` → `lloyd_max(bits-1, 2048, ...)`

### Uniform 1K x 1536 x 4-bit

| Variant | Recall@10 | NDCG@10 | MAE |
|---|---:|---:|---:|
| `prod_cb512` | `71.0%` | `0.8037` | `0.002851` |
| `prod_cb1536` | `77.0%` | `0.8565` | `0.002378` |
| `prod_cb2048` | `80.5%` | `0.8742` | `0.003659` |

### Clustered 1K x 1536 x 4-bit

| Variant | Recall@10 | NDCG@10 | MAE |
|---|---:|---:|---:|
| `prod_cb512` | `72.5%` | `0.7967` | `0.002696` |
| `prod_cb1536` | `81.5%` | `0.8746` | `0.002037` |
| `prod_cb2048` | `79.5%` | `0.8645` | `0.003526` |

## Full-`n` Exact-Only Reference Check

I also reran the existing full-tail exact-only reference on uniform data and added the clustered
counterpart to test the remaining upstream architectural gap more directly.

### Uniform 1K x 1536 x 4-bit

| Variant | Recall@10 | NDCG@10 | MAE |
|---|---:|---:|---:|
| `current_exact` | `77.0%` | `0.8565` | `0.002378` |
| `tail_full_cb1536` | `83.0%` | `0.8955` | `0.002043` |
| `tail_full_cb2048` | `81.0%` | `0.8787` | `0.002370` |

### Clustered 1K x 1536 x 4-bit

| Variant | Recall@10 | NDCG@10 | MAE |
|---|---:|---:|---:|
| `current_exact` | `81.5%` | `0.8746` | `0.002037` |
| `tail_full_cb1536` | `81.5%` | `0.8802` | `0.002189` |
| `tail_full_cb2048` | `80.5%` | `0.8729` | `0.002326` |

## Readout

### Failed Primary Fix: `tile_dim = 512` codebook

This is now a recorded failed lane.

- It regresses Recall@10 on both corpora
- It regresses NDCG@10 on both corpora
- It increases MAE on both corpora

So the review-204 recommendation to "fix codebook parameterization immediately" by switching tiled
FWHT to `tile_dim` is not supported by current measurements.

### Mixed Alternative: `transform_dim = 2048` codebook

This remains a live but weaker hypothesis:

- it helps on uniform `1k` (`77.0% -> 80.5%`)
- it hurts on clustered `1k` (`81.5% -> 79.5%`)
- it increases MAE materially on both corpora

So `cb2048` is not a safe drop-in production default either. It is dataset-sensitive and likely
needs broader evidence before any code change.

### What Actually Survived Review 204

The strong part of review `204` still holds:

- TurboQuantDB's major difference is not "tile-dim codebook tuning"
- it is the full-`n` architecture: full padded transform, encode all `n`, score all `n`

The failed part is the shortcut conclusion that tiled FWHT should immediately switch to a
`tile_dim` codebook.

The new full-`n` reference result sharpens that further:

- full-`n` is a real quality lever on uniform `1k` (`77.0% -> 83.0%`)
- but it is not a universal Recall@10 fix on clustered `1k` (`81.5% -> 81.5%`)
- on clustered data it helps rank quality more than set overlap (`NDCG@10 0.8746 -> 0.8802`)

## Current Hypothesis Ranking

1. `H1: graph-runtime gap`
   - secondary after the tiled-FWHT repair
2. `H2: quantized-objective mismatch`
   - still dominant
3. `H3: quantized-path implementation defect`
   - narrowed: the live defect is no longer obviously "wrong codebook dim for tiled path"

More precise sub-read:

- `H3b: tiled path should use tile_dim codebook` — downgraded; failed as a primary fix
- `H3c: current 1536 tiled path still differs too much from upstream full-n architecture` —
  still live

## Recommended Next Step

Do not change production from `cb1536` to `cb512`.

The next efficient investigation should be one of:

1. a full-`n` exact-only reference closer to TurboQuantDB's real architecture, using the current
   tqvector scorer/evidence harness
2. a broader structured-data comparison of `cb1536` vs `cb2048` if we want to know whether the
   mixed result is just a synthetic-dataset artifact

If the goal is the fastest path to new signal, option 1 is better: it tests the actual upstream
architectural difference instead of continuing to tune a codebook parameter that already failed one
direct measurement. The new `1k` full-`n` result also says to expect a nuanced answer, not a
single silver bullet.

## Commands Run

```bash
cargo test --test recall_integration quantizer_recall_1536_tiled_fwht_reference_1k_clustered -- --ignored --nocapture
cargo test --test recall_integration quantizer_recall_1536_padding_ablations_1k_clustered -- --ignored --nocapture
cargo test --test recall_integration quantizer_recall_1536_padding_ablations_1k_uniform -- --ignored --nocapture
cargo test --test recall_integration quantizer_recall_1536_tiled_codebook_dim_sweep_1k_clustered -- --ignored --nocapture
cargo test --test recall_integration quantizer_recall_1536_tiled_codebook_dim_sweep_1k_uniform -- --ignored --nocapture
cargo test --test recall_integration quantizer_recall_1536_full_tail_exact_1k_uniform -- --ignored --nocapture
cargo test --test recall_integration quantizer_recall_1536_full_tail_exact_1k_clustered -- --ignored --nocapture
cargo test nearest_centroid_index_prefers_lower_index_on_tie -- --nocapture
```

## Review Focus

- whether the new codebook-dimension sweeps are sufficient to retire the `cb512` shortcut
- whether the next reference should be full-`n` exact-only before any more production quantizer
  changes
- whether the mixed `cb2048` result changes how we interpret the remaining gap to TurboQuantDB
