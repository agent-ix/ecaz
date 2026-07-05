# Review Request: C1 ADR-031 On/Off A/B

## Context

Packet `288` established two things on the current Tier 1 build:

1. the high-`ef_search` frontier is still fast on the full real `50k` table
2. a fair `queries_50` comparison against the old A4 seam shows lower Recall@10
   than the earlier A4-era slice

That leaves the main diagnostic question unresolved:

- is the high-`ef_search` recall drop actually caused by ADR-031, or
- is it from later non-ADR-031 evolution in the graph/runtime/index state?

## Problem

The current build has a same-build toggle for persisted sidecar usage
(`tqhnsw.force_binary_derivation`), but it does **not** yet have a same-build
way to disable ADR-031 runtime behavior entirely.

Without that seam, we can compare old packets to new packets, but we cannot do
the decisive test:

- same codebase
- same fixture
- same query table
- same `ef_search`
- ADR-031 fully enabled vs fully disabled

## Planned Slice

Add the smallest hidden diagnostic seam that disables ADR-031 runtime behavior
entirely by skipping binary-query preparation during scan setup.

That should turn off both:

- binary-sign prefilter scoring
- ADR-031-driven lazy exact scoring on cache miss

while leaving the rest of the current build untouched.

Then run the current build with ADR-031:

1. enabled
2. disabled

on the same high-`ef_search` recall seams.

## Success Criteria

- the packet records the exact on/off switch used
- the packet records same-build recall results for ADR-031 enabled vs disabled
  on the same real-corpus seam
- the packet makes a clear call on whether ADR-031 is the cause of the
  high-`ef_search` recall drop

## Code Checkpoint

The diagnostic seam landed in commit `7580dee`:

- new hidden GUC: `tqhnsw.disable_binary_prefilter`
- scan setup skips ADR-031 binary-query preparation when that GUC is enabled
- the existing runtime otherwise stays unchanged

Validation on the checkpoint was green:

- `cargo test`
- `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

## Toggle Used

ADR-031 enabled:

```bash
./scripts/pg17_scratch_psql.sh --sql "
  ALTER DATABASE postgres RESET tqhnsw.disable_binary_prefilter;
  ALTER DATABASE postgres RESET tqhnsw.force_binary_derivation;
"
```

ADR-031 disabled:

```bash
./scripts/pg17_scratch_psql.sh --sql "
  ALTER DATABASE postgres SET tqhnsw.disable_binary_prefilter = on;
  ALTER DATABASE postgres RESET tqhnsw.force_binary_derivation;
"
```

Each recall run uses a fresh scratch `psql` session, so the database-level GUC
switch is sufficient for a same-build A/B.

## Apples-to-Apples `queries_50` Comparison

Enabled runs on the current build:

```bash
./scripts/run_real_corpus_recall_scratch.sh summary \
  --index tqhnsw_real_50k_m8_idx \
  --m 8 \
  --ef-search 128 \
  --corpus-table tqhnsw_real_50k_corpus \
  --queries-table tqhnsw_real_50k_queries_50
```

```bash
./scripts/run_real_corpus_recall_scratch.sh summary \
  --index tqhnsw_real_50k_m8_idx \
  --m 8 \
  --ef-search 200 \
  --corpus-table tqhnsw_real_50k_corpus \
  --queries-table tqhnsw_real_50k_queries_50
```

Observed enabled outputs:

```text
8  128  50000  50  0.89   0.8734  0.92889374  0.0055691516  0.7583029   0.86  1  1
8  200  50000  50  0.894  0.8944  0.9313327   0.0055674054  0.76424235  0.86  1  1
```

Disabled runs on the same build:

```bash
./scripts/run_real_corpus_recall_scratch.sh summary \
  --index tqhnsw_real_50k_m8_idx \
  --m 8 \
  --ef-search 128 \
  --corpus-table tqhnsw_real_50k_corpus \
  --queries-table tqhnsw_real_50k_queries_50
```

```bash
./scripts/run_real_corpus_recall_scratch.sh summary \
  --index tqhnsw_real_50k_m8_idx \
  --m 8 \
  --ef-search 200 \
  --corpus-table tqhnsw_real_50k_corpus \
  --queries-table tqhnsw_real_50k_queries_50
```

Observed disabled outputs:

```text
8  128  50000  50  0.89   0.8734  0.92889374  0.0055691516  0.7583029   0.86  1  1
8  200  50000  50  0.894  0.8944  0.9313327   0.0055674054  0.76424235  0.86  1  1
```

So the apples-to-apples A4 seam is bit-for-bit identical with ADR-031 enabled
and disabled.

## Canonical Full-Table Confirmation

To make sure the result was not limited to the `50`-query fairness slice, I
also reran the disabled build on the full canonical table:

```bash
./scripts/run_real_corpus_recall_scratch.sh summary \
  --index tqhnsw_real_50k_m8_idx \
  --m 8 \
  --ef-search 128 \
  --corpus-table tqhnsw_real_50k_corpus \
  --queries-table tqhnsw_real_50k_queries
```

```bash
./scripts/run_real_corpus_recall_scratch.sh summary \
  --index tqhnsw_real_50k_m8_idx \
  --m 8 \
  --ef-search 200 \
  --corpus-table tqhnsw_real_50k_corpus \
  --queries-table tqhnsw_real_50k_queries
```

Observed disabled outputs:

```text
8  128  50000  1000  0.8977  0.85971  0.9341158  0.006018223  0.77886665  0.8428  9  1
8  200  50000  1000  0.9039  0.88845  0.938355   0.006021826  0.79619354  0.8428  9  1
```

These are also bit-for-bit identical to packet `288`'s enabled canonical
results on the same build line.

## Readout

The A/B answer is decisive:

- disabling ADR-031 does not change the `queries_50` fairness seam at
  `ef_search=128` or `200`
- disabling ADR-031 does not change the full canonical `1000`-query seam at
  `ef_search=128` or `200`

So ADR-031 is **not** the source of the higher-`ef_search` recall shift versus
the older A4-era records. The quality delta comes from some later non-ADR-031
change in the graph/runtime/index state or in the exact fixture/index history
behind those earlier packets.

That clears ADR-031 Tier 2 as the correct next move. The next optimization
seam can focus on:

1. pin-and-hold graph element reads
2. exact scoring directly from borrowed `TqElementTupleRef.code`
3. deleting the remaining `element.code.to_vec()` copy on the ADR-031 hot path
