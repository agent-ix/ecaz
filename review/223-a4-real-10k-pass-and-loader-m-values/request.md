# Review Request: A4 Real 10k Pass + Loader `--m` Fix

## Context

Branch:
- `main`

Prior real-corpus packets:
- `review/218-a4-real-corpus-recall-lane/request.md`
- `review/219-a4-real-corpus-loader-smoke/request.md`
- `review/220-a4-real-corpus-metric-contract-followup/request.md`
- `review/221-a4-real-corpus-subset-manifest-contract/request.md`
- `review/222-a4-real-corpus-fetch-and-schema-alignment/request.md`

This slice does two things:

1. records the first successful real-data A4 gate result on the canonical
   `10k` DBpedia-derived subset
2. fixes a real loader bug discovered while moving from the `10k` proof point
   to the default `50k` working subset

## What Landed

### 1. Real `10k` A4 gate passes strongly

On the canonical real `10k` subset:

- corpus: `tqhnsw_real_10k_corpus` (`10,000` rows)
- queries: `tqhnsw_real_10k_queries` (`200` rows)
- index set: `tqhnsw_real_10k_m8_idx`, `tqhnsw_real_10k_m16_idx`

The live graph-first external gate report returned:

```text
8   40   0.971         t
8   128  0.973  0.89   t
8   200  0.974         t
16  200  0.975         t
```

So the A4 gate at `(m=8, ef_search=128)` is not merely passing; it is passing
comfortably at `97.3%` Recall@10 on the real `10k` DBpedia-derived subset.

### 2. Real `10k` exact-vs-graph gap is small

The smaller real summary slice on the same `10k` corpus at the gate point
`(m=8, ef_search=128)` over `50` fixed real queries returned:

```text
8  128  10000  50  0.972  0.971  0.9826096  0.009107447  0.9689694  0.976  1  2
```

Interpreted:

- graph Recall@10: `0.972`
- graph Recall@100: `0.971`
- exact quantized Recall@10: `0.976`
- graph-below-exact queries: `1`
- worst exact gap: `2`

That means the live graph path is only `0.4pp` below exact quantized on real
`10k`. This materially weakens the old synthetic-only read that graph/runtime
was the primary blocker.

### 3. The loader had a real `--m` parsing bug

While moving to the real `50k` subset, the loader exposed a bug in
`scripts/load_real_corpus.py`:

- documented usage allowed either `--m 8 16` or repeated `--m 8 --m 16`
- actual argparse behavior only honored the last repeated `--m` occurrence
- so `--m 8 --m 16` incorrectly built only `m=16`

This was the direct cause of the wasted first `50k` index-build attempt. We
were paying to build the non-gate-critical `m=16` index first instead of the
threshold config `m=8`.

The fix now:

- accepts both `--m 8 16` and repeated `--m 8 --m 16`
- flattens repeated groups into one deduplicated ordered list
- logs the resolved `m=[...]` list in the final loader output

## Evidence

### Real `10k` gate pass

Observed output from:

```sql
select * from tests.tqhnsw_graph_scan_recall_external_gate_report(
    'tqhnsw_real_10k_corpus',
    'tqhnsw_real_10k_queries',
    'tqhnsw_real_10k'
);
```

was:

```text
8   40   0.971         t
8   128  0.973  0.89   t
8   200  0.974         t
16  200  0.975         t
```

### Real `10k` gate-point summary

Observed output from:

```sql
select * from tests.tqhnsw_graph_scan_recall_external_summary(
    'tqhnsw_real_10k_corpus',
    'tqhnsw_real_10k_queries_50',
    'tqhnsw_real_10k_m8_idx',
    8,
    128
);
```

was:

```text
8  128  10000  50  0.972  0.971  0.9826096  0.009107447  0.9689694  0.976  1  2
```

### Failed path: repeated `--m` flags built the wrong index

Before the fix, the real `50k` loader invocation:

```bash
./scripts/load_real_corpus_scratch.sh \
    --prefix tqhnsw_real_50k \
    --corpus-file .../tqhnsw_real_50k_corpus.tsv \
    --queries-file .../tqhnsw_real_50k_queries.tsv \
    --m 8 --m 16
```

went straight into:

```text
[loader] building tqhnsw_real_50k_m16_idx (m=16, ef_construction=128) ...
```

That was the wrong priority for A4 because the actual threshold gate is
`(m=8, ef_search=128)`.

### Repeated-flag smoke after the fix

After the fix, the same repeated-flag form against the already-built real `10k`
fixture returned:

```text
[loader] verified manifest /home/peter/dev/datasets/tqhnsw_real_10k/tqhnsw_real_10k_manifest.json for prefix tqhnsw_real_10k
[loader] tqhnsw_real_10k_corpus already has 10000 rows; skipping reload
[loader] tqhnsw_real_10k_queries already has 200 rows; skipping reload
[loader] tqhnsw_real_10k_m8_idx already exists with m=8 ef_construction=128; skipping rebuild
[loader] tqhnsw_real_10k_m16_idx already exists with m=16 ef_construction=128; skipping rebuild
[loader] done. corpus=tqhnsw_real_10k_corpus (10000 rows), queries=tqhnsw_real_10k_queries (200 rows), m=[8, 16]
```

That is the intended contract.

## What Did Not Work

### Full `50k` exact-vs-graph summary is too expensive for iteration

The current SQL summary surface
`tests.tqhnsw_graph_scan_recall_external_summary(...)` is still too heavy at
`50k` even with a `50`-query slice. The backend stayed pure CPU for ~30 minutes
with no lock or I/O wait, because the path still pays for:

- full fp32 brute-force truth
- full exact quantized top-10
- full live graph scan

for the same query set inside one wrapper.

This is now a harness concern, not a correctness contradiction. Real `10k`
already shows the current graph path is essentially tracking exact quantized on
real data.

## Readout

The important conclusion from this slice is:

- on real DBpedia-derived `10k`, A4 already passes strongly
- the live graph path is close to exact quantized on real data
- the next blocker is scale-measurement cost on `50k`, not the old synthetic
  recall contradiction

That means the next useful harness work is to make the real `50k` exact-vs-graph
diagnostics cheaper, not to reopen quantizer/runtime debugging blindly.

## Files

- `scripts/load_real_corpus.py`
