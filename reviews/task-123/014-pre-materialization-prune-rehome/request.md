# Task 123 Review Request: Pre-Materialization Prune Rehome

## Scope

This packet re-homes the SPIRE pre-materialization prune code from draft PR #43
onto the Task 121/123 branch so the reopened multi-instance latency +
communications mandate can A/B it on the correct substrate.

Code commits added before this packet:

- `a5d03abc1` - `Skip doomed SPIRE candidate materialization`
- `ea036b542` - `Prune SPIRE batched TQ materialization`
- `8bdbf7b0b` - `Gate SPIRE pre-materialization prune`

No Task 122 historical benchmark packets were copied. Task 123 will produce its
own contained local multi-instance evidence.

## Behavior

The code adds `ec_spire.pre_materialization_prune`, defaulting on, and skips
SPIRE V2 row materialization when the bounded, non-deduped scan already has a
top-k keep threshold and the quantized candidate score cannot enter it. Skipped
rows are counted through the existing truncated-candidate diagnostics.

The guard remains intentionally narrow:

- bounded scan only;
- replica dedupe disabled;
- no delete-delta vec_ids requiring materialization before filtering;
- finite candidate score;
- candidate score below the current keep threshold.

## Why This Belongs In Task 123

The packet 013 reviewer accepted the closeout-decline response and noted that
the payload cap is the communications question. PR #43's prune is the matching
candidate-count lever: it can reduce how many candidates reach tuple
materialization and remote tuple shipping. The prior PR #43 evidence was
single-instance, so Task 123 must remeasure it on the contained local
multi-instance executor.

## Validation

Focused PG18 diagnostics passed:

```text
cargo test -p ecaz --lib --no-default-features --features pg18 collect_scan_placement_diagnostics -- --nocapture
```

Packet-local log:

- `artifacts/cargo-test-spire-diagnostics.log`

Key result:

```text
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 2223 filtered out
```

## Next Measurement

Next packet should run the contained local multi-instance communications pass
against the selected `n1024 b2/tr50/f8` surface, bracketing:

- raised-cap `id,source` payload;
- narrow payload;
- prune on/off via `ec_spire.pre_materialization_prune`.
