# Review Request: A4 10K Operating-Point Triage

Basis: `main` working tree after review `205`

## Why This Packet Exists

Review `205` narrowed the remaining quantizer questions, but the strongest evidence was still on
`1k` corpora. This packet reruns the highest-value exact-only probes on clustered `10k` to answer
three concrete questions:

1. Is the remaining upstream full-`n` mismatch still a big driver at `10k`?
2. Is the current `4-bit` operating point itself the dominant ceiling?
3. Does the ADR-021 payload-equivalent workaround (`2048-full-n @ 3-bit`) actually help?

## New 10K Exact-Only Experiments

New ignored tests were added in [recall_integration.rs](/home/peter/dev/tqvector/tests/recall_integration.rs):

- `quantizer_recall_1536_upstream_gap_10k_clustered`
- `quantizer_recall_clustered_10k_bitwidth_spot_check`
- `quantizer_recall_1536_payload_equivalent_operating_points_10k_clustered`

All three use the same fixed clustered synthetic corpus:

- `10,000` indexed vectors
- `50` query vectors
- `1536` dimensions
- `50` clusters
- spread `0.3`
- seed `42`

Ground truth remains brute-force fp32 top-k.

## Result 1: Upstream Full-`n` Gap at 10K

### Clustered 10K x 1536

| Variant | Recall@10 | NDCG@10 | MAE |
|---|---:|---:|---:|
| `current_exact` | `74.8%` | `0.8201` | `0.002263` |
| `prod_cb2048` | `74.0%` | `0.8151` | `0.003585` |
| `tail_full_cb1536` | `74.8%` | `0.8267` | `0.001777` |
| `tail_full_cb2048` | `76.0%` | `0.8331` | `0.002217` |

### Readout

- Switching the tiled production path to `cb2048` alone regresses at `10k`
- Full-`n` with the current `1536` codebook helps rank quality and top-1 behavior, but not
  Recall@10 set overlap
- Full-`n` plus `cb2048` is the best of the four, but only by `+1.2pp` Recall@10

So the remaining upstream architectural mismatch is real but secondary on the clustered `10k`
corpus. It is not large enough to explain the entire A4 miss.

## Result 2: Bit Budget at 10K

### Clustered 10K x 1536, Current Production Path

| Bits | Recall@10 | NDCG@10 | MAE |
|---|---:|---:|---:|
| `3` | `54.2%` | `0.6076` | `0.005242` |
| `4` | `74.8%` | `0.8201` | `0.002263` |
| `6` | `92.0%` | `0.9509` | `0.000831` |
| `8` | `97.0%` | `0.9826` | `0.000258` |

### Readout

This is the biggest new fact in the lane.

- The `89%` gate is not impossible in principle on this corpus
- It is impossible on the current `1536 @ 4-bit` operating point
- The same quantizer family crosses the gate cleanly at `6-bit`

So the dominant ceiling on clustered `10k` is now the `4-bit` rate, not graph traversal and not
the remaining full-`n` mismatch.

## Result 3: Payload-Equivalent ADR-021 Check

ADR-021 proposed that `2048-full-n @ 3-bit` might be a viable same-payload alternative to
`1536 @ 4-bit`. This packet checks that directly.

### Clustered 10K x 1536

| Variant | Recall@10 | NDCG@10 | MAE |
|---|---:|---:|---:|
| `current_1536_4bit` | `74.8%` | `0.8201` | `0.002263` |
| `fulln_2048_3bit` | `58.6%` | `0.6748` | `0.006496` |
| `current_1536_6bit` | `92.0%` | `0.9509` | `0.000831` |

### Readout

This retires the payload-equivalent escape hatch for this corpus.

- `2048-full-n @ 3-bit` is materially worse than current `1536 @ 4-bit`
- the same-payload argument does not rescue recall here
- the first operating point that actually clears the gate is `1536 @ 6-bit`

## Compound-Problem Read

Yes, the A4 miss is compound. The evidence now looks like this:

1. `1536 -> 2048` truncation was a major early defect and tiled FWHT fixed it
2. full-`n` architecture is a real but modest improvement on `10k`
3. codebook-dimension shortcuts (`cb512`, `cb2048` on tiled path) are not primary fixes
4. the dominant remaining ceiling on clustered `10k` is the `4-bit` operating point itself

So isolated tests were still necessary, but not because any one of them was expected to prove the
final fix. Their job was to rank factors. The factor ranking is now much clearer.

## Updated Hypothesis Ranking

1. `H1: graph-runtime gap`
   - secondary relative to the quantized ceiling on `10k`
2. `H2: quantized-objective mismatch`
   - dominant, now narrowed further to the current `4-bit` operating point
3. `H3: quantized-path implementation defect`
   - no longer the leading explanation for the remaining gap

More precise sub-read:

- `H3b: tiled path should use tile_dim codebook` — failed
- `H3c: remaining full-n mismatch is the main blocker` — weakened; only `+1.2pp` at `10k`
- `H2b: 4-bit rate is too lossy for this clustered `10k` gate` — now strongly supported

## Recommended Next Step

Do not spend the next slice on more `4-bit` codebook tuning.

The clean next options are:

1. if the product can absorb the payload increase, test the live graph/runtime path at `6-bit`
   because exact-only evidence says that is the first operating point that clears the gate
2. if payload must stay near current size, stop assuming ADR-021's `2048@3bit` workaround is
   viable on this corpus; it is not
3. only continue full-`n` architecture work if we care about incremental quality gains, not as the
   primary path to clearing A4

## Commands Run

```bash
cargo test --test recall_integration quantizer_recall_1536_upstream_gap_10k_clustered -- --ignored --nocapture
cargo test --test recall_integration quantizer_recall_clustered_10k_bitwidth_spot_check -- --ignored --nocapture
cargo test --test recall_integration quantizer_recall_1536_payload_equivalent_operating_points_10k_clustered -- --ignored --nocapture
```

## Review Focus

- whether the `10k` results are strong enough to demote full-`n` work from primary blocker to
  secondary quality improvement
- whether the lane should now pivot from architecture/debug work to operating-point work
- whether any same-payload alternative still looks credible after `2048-full-n @ 3-bit` failed
