# Review Request: A4 1536 4-bit QJL-vs-MSE Reallocation

Basis: `main` working tree after review `206`

## Why This Packet Exists

Review `206` established that clustered `10k` Recall@10 is now dominated by the current
`1536 @ 4-bit` operating point:

- `1536 @ 4-bit`: `74.8%`
- `1536 @ 6-bit`: `92.0%`

That still left one important same-payload question open:

At `1536 @ 4-bit`, is the remaining loss coming from the rate itself, or from how the current path
spends that rate (`3` MSE bits + `1` QJL bit)?

This packet tests that directly.

## Code Change Under Test

The current working tree changes the tiled `1536 @ 4-bit` production path from:

- `3` MSE bits + `1` QJL bit

to:

- `4` MSE bits + `0` QJL bits

at the same stored payload size:

- `4` gamma bytes
- `768` packed MSE bytes
- `0` QJL bytes
- total payload still `772` bytes

The production change is in:

- [prod.rs](/home/peter/dev/tqvector/src/quant/prod.rs)
- [mod.rs](/home/peter/dev/tqvector/src/quant/mod.rs)
- [size_of_assertions.rs](/home/peter/dev/tqvector/tests/size_of_assertions.rs)

Key implementation read:

- QJL is now disabled specifically for tiled `1536 @ 4-bit`
- the freed bit is reallocated to MSE quantization
- payload size stays constant, so this is a true same-byte operating-point comparison

## New Exact-Only Experiment

New ignored test in [recall_integration.rs](/home/peter/dev/tqvector/tests/recall_integration.rs):

- `quantizer_recall_1536_same_payload_qjl_vs_mse_10k_clustered`

Corpus and ground truth match review `206`:

- `10,000` indexed vectors
- `50` query vectors
- `1536` dimensions
- `50` clusters
- spread `0.3`
- seed `42`
- brute-force fp32 top-k truth

The comparison is:

- `legacy_3mse_plus_qjl`
  - tiled-FWHT exact-only reference matching the old `3+1` split
- `current_4mse_no_qjl`
  - the new production path on the current tree

## Results

### Clustered 10K x 1536, Same Payload

| Variant | Recall@10 | NDCG@10 | MAE |
|---|---:|---:|---:|
| `legacy_3mse_plus_qjl` | `74.8%` | `0.8201` | `0.002263` |
| `current_4mse_no_qjl` | `84.8%` | `0.9037` | `0.001262` |

## Readout

This is the biggest same-payload improvement since the tiled-FWHT repair.

- Recall@10 improves by `+10.0pp`
- NDCG@10 improves by `+0.0836`
- MAE drops by about `44%`

So the thin-QJL allocation was materially harmful at tiled `1536 @ 4-bit`.

This does **not** prove that QJL is useless in general. It does show something narrower and more
actionable:

- one QJL sign bit is a poor use of the final bit at this operating point
- the current `3+1` split was leaving too much value on the table
- the `4-bit` ceiling from review `206` was partly a bit-allocation defect, not just a pure rate
  limit

## What This Changes About The Hypotheses

Updated read relative to review `206`:

1. `H1: graph-runtime gap`
   - still needs rerun on the live path, but exact-only headroom improved materially
2. `H2: quantized-objective mismatch`
   - still real, but weaker than it looked under the old `3+1` split
3. `H3: quantized-path implementation defect`
   - strengthened again in a narrower form:
   - `H3d: tiled 1536 @ 4-bit spends its final bit on the wrong signal`

So the new evidence does not overturn the compound-problem read. It sharpens it:

- tiled FWHT fixed the early `1536 -> 2048` truncation loss
- `4+0` fixes a second major `1536 @ 4-bit` operating-point defect
- whatever A4 gap remains after this should be measured again before assuming `4-bit` itself is
  fundamentally hopeless

## Recommended Next Step

Do not spend the next slice on QR or more upstream-paper parity work.

The next efficient sequence is:

1. validate the new production path fully
2. rerun the cheap live `1k` graph probe on the new `4+0` path
3. if `1k` still looks sane, rerun the real A4 gate on clustered `10k`

If the live graph path now tracks the improved exact-only ceiling closely enough, A4 may be back in
range without increasing the payload.

## Commands Run

```bash
cargo test --test recall_integration quantizer_recall_1536_same_payload_qjl_vs_mse_10k_clustered -- --ignored --nocapture
cargo test quantizer_1536_4bit_reallocates_qjl_budget_to_mse -- --nocapture
cargo test --no-default-features --features 'pg17 pg_test' tests::pg_test_tqhnsw_graph_scan_recall_fixture_summary_1k_tiled_fwht -- --exact --ignored --nocapture
```

## Review Focus

- whether the same-payload `+10pp` exact-only gain is strong enough to treat the old `3+1` split
  as a failed lane
- whether the production special-case for tiled `1536 @ 4-bit` is an acceptable policy surface
- whether the next highest-value measurement is now the live A4 rerun rather than more offline
  `4-bit` theory work
