# Review Request: A4 4-MSE Codebook Sweep on Current 4+0 Path

Basis: `main` working tree after review `208`

## Why This Packet Exists

Review `208` demoted thin QJL on the current small-storage lane:

- `current_4mse_no_qjl`: `84.8%` Recall@10
- `4mse_plus_qjl`: `84.8%`

That leaves a cleaner next question inside the same-byte `4+0` lane:

Is the current `4+0` ceiling still partly a codebook-parameterization problem, or is the
production `cb1536` choice already the best of the obvious zero-cost alternatives?

This packet answers that directly on clustered `10k`.

## New Exact-Only Experiment

New ignored test in [recall_integration.rs](/home/peter/dev/tqvector/tests/recall_integration.rs):

- `quantizer_recall_1536_4mse_codebook_dim_sweep_10k_clustered`

The helper in the same file explicitly mirrors the current production MSE-bit policy:

- tiled `1536 @ 4-bit` uses `4` MSE bits, not `bits - 1`
- only the analytic codebook dimension is varied

Corpus and truth match recent A4 quantizer probes:

- `10,000` indexed vectors
- `50` query vectors
- `1536` dimensions
- `50` clusters
- spread `0.3`
- seed `42`
- brute-force fp32 top-k truth

Variants:

1. `current_cb1536`
   - current production `4+0` path
   - `lloyd_max(4, 1536, ...)`

2. `4mse_cb512`
   - tiled-path codebook parameterized by tile size
   - `lloyd_max(4, 512, ...)`

3. `4mse_cb1536`
   - explicit control copy of current production behavior

4. `4mse_cb2048`
   - padded-transform prior
   - `lloyd_max(4, 2048, ...)`

## Results

### Clustered 10K x 1536

| Variant | Recall@1 | Recall@10 | NDCG@10 | MAE |
|---|---:|---:|---:|---:|
| `current_cb1536` | `86.0%` | `84.8%` | `0.9037` | `0.001262` |
| `4mse_cb512` | `70.0%` | `78.8%` | `0.8614` | `0.001742` |
| `4mse_cb1536` | `86.0%` | `84.8%` | `0.9037` | `0.001262` |
| `4mse_cb2048` | `72.0%` | `84.2%` | `0.8984` | `0.001671` |

## Readout

This is a clean ranking result.

- current production `cb1536` remains the best variant on clustered `10k`
- `cb512` is not a near miss; it is a strong regression
- `cb2048` comes close on Recall@10 but still loses to current production and regresses Recall@1,
  NDCG@10, and MAE

So the main codebook-dimension alternatives are now much weaker on the current `4+0` path than
they looked in earlier pre-`207` discussion.

The practical read:

- `cb512` should stay demoted as a failed primary fix
- `cb2048` is not strong enough to justify a production switch
- the current `4+0 + cb1536` path is not leaving an obvious zero-cost codebook win on the table

## What This Changes

Combined with reviews `207` and `208`, the `4+0` lane now looks like this:

1. reallocating the final bit from thin QJL to MSE was a major win
2. adding thin QJL back on top of strong MSE is effectively flat
3. retuning the analytic codebook to `512` or `2048` does not improve the current path

That means the next same-byte improvement is unlikely to come from another simple codebook-dim
swap.

## Recommended Next Step

Do not spend the next slice on `cb512` or `cb2048`.

The better options now are:

1. identify a different same-byte `4+0` quality lever, or
2. make the live `10k` A4 harness practical enough to measure the real graph/runtime path again

If the goal is fastest certainty, the harness path is becoming more attractive because the obvious
exact-only `4+0` codebook alternatives are now ruled down.

## Commands Run

```bash
cargo test --test recall_integration quantizer_recall_1536_4mse_codebook_dim_sweep_10k_clustered -- --ignored --nocapture
```

## Review Focus

- whether the clustered `10k` sweep is strong enough to demote `cb512` and `cb2048` on the current
  `4+0` lane
- whether the next slice should now pivot from offline codebook hypotheses back to harness/runtime
  practicality
- whether any other same-byte `4+0` hypothesis still looks stronger than a live A4 rerun
