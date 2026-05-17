# Review Request: A4 Thin-QJL Marginal Value After 4 MSE Bits

Basis: `main` working tree after review `207`

## Why This Packet Exists

Review `207` established a strong same-payload result on clustered `10k`:

- old `3 MSE + 1 QJL` split: `74.8%` Recall@10
- current `4 MSE + 0 QJL` split: `84.8%` Recall@10

That moved the lane decisively toward `4+0`, but it still left one important question open:

Does thin QJL become useful again once MSE is already strong, or is it still basically inert even
after we give MSE its fourth bit?

This packet answers that directly.

## New Exact-Only Experiment

New ignored test in [recall_integration.rs](/home/peter/dev/tqvector/tests/recall_integration.rs):

- `quantizer_recall_1536_qjl_increment_after_4mse_10k_clustered`

Corpus and truth match reviews `206` and `207`:

- `10,000` indexed vectors
- `50` query vectors
- `1536` dimensions
- `50` clusters
- spread `0.3`
- seed `42`
- brute-force fp32 top-k truth

The comparison is:

1. `current_4mse_no_qjl`
   - current production `1536 @ 4-bit`
   - `4` MSE bits, `0` QJL bits
   - `772` bytes total payload

2. `4mse_plus_qjl_g0`
   - `bits=5` production path, which gives `4` MSE bits + `1` QJL bit
   - but scored with `gamma = 0`
   - isolates the strong-MSE baseline with QJL disabled at score time

3. `4mse_plus_qjl`
   - same `bits=5` path
   - full scorer with the thin residual/QJL term enabled

The key read is:

- `4mse_plus_qjl - 4mse_plus_qjl_g0` isolates thin-QJL's marginal value once MSE already has
  four bits
- `4mse_plus_qjl_g0 - current_4mse_no_qjl` checks whether the stronger-MSE baseline is already
  fully captured by current `4+0`

## Results

### Clustered 10K x 1536

| Variant | Recall@1 | Recall@10 | NDCG@10 | MAE |
|---|---:|---:|---:|---:|
| `current_4mse_no_qjl` | `86.0%` | `84.8%` | `0.9037` | `0.001262` |
| `4mse_plus_qjl_g0` | `86.0%` | `84.8%` | `0.9037` | `0.001262` |
| `4mse_plus_qjl` | `86.0%` | `84.8%` | `0.9041` | `0.001236` |

## Readout

This is a very strong negative result for thin QJL on the current formulation.

- giving MSE its fourth bit already captures essentially all of the quality improvement
- enabling the thin QJL term on top of that does not move Recall@10 at all on this corpus
- the only visible change is a tiny NDCG/MAE nudge

So the current thin-QJL path is still not a primary quality lever even after MSE reconstruction is
substantially stronger.

More concretely:

- `4mse_plus_qjl_g0` exactly matches `current_4mse_no_qjl`
- `4mse_plus_qjl` is numerically almost identical for Recall@10
- the thin residual/QJL term is therefore not where the next large recall gain lives

## What This Changes

This sharpens the post-`207` read further:

1. `4+0` remains the highest-ROI small-storage lane
2. thin QJL is now a recorded low-value follow-up, not the next default bet
3. if QJL comes back later, it should be as a materially different formulation, not as a reason to
   undo the `4+0` reallocation

This does **not** prove that every richer QJL design is useless. It does prove something much more
useful for the current repo:

- the existing thin QJL term is not competitive with one more MSE bit at `1536`
- even when MSE already has four bits, the thin QJL add-on stays near-zero for Recall@10

## Runtime Note

I also reran the cheap live `1k` tiled fixture guard after landing `207`.

- it still passes on the real graph-first path

I then started the real live A4 gate rerun again, but the harness still spent multiple minutes in
`pg_stat_progress_create_index` phase `building index` with no visible progress counters before it
was stopped to keep the lane usable.

That does not invalidate the quantizer result above. It just means the live `10k` harness is still
too expensive to use as the primary inner loop.

## Recommended Next Step

Do not pivot back to thin QJL work.

The next efficient options are:

1. keep exploring same-byte `4+0` improvements
2. optimize the live `10k` recall harness so A4 can be rerun without burning a whole turn on index
   build
3. defer fuller-QJL work until we have exhausted the cleaner `4+0` lane

## Commands Run

```bash
cargo test --test recall_integration quantizer_recall_1536_qjl_increment_after_4mse_10k_clustered -- --ignored --nocapture
cargo test --no-default-features --features 'pg17 pg_test' tests::pg_test_tqhnsw_graph_scan_recall_fixture_summary_1k_tiled_fwht -- --exact --ignored --nocapture
TQVECTOR_RUN_RECALL_GATE=1 cargo test --no-default-features --features 'pg17 pg_test' tests::pg_test_tqhnsw_graph_scan_recall_gate -- --exact --nocapture
```

## Review Focus

- whether the new `4mse_plus_qjl` ablation is strong enough to demote thin QJL from a live recall
  hypothesis
- whether the repo should treat `4+0` as the default small-storage lane until proven otherwise
- whether the next bottleneck is now clearly harness/runtime cost rather than quantizer ambiguity
