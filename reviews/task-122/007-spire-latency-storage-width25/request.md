# Task 122 Packet 007: SPIRE Width-25 Latency and Storage

This packet follows packet 006's recall-width sweep with a release latency and
storage matrix at width `25`. Packet 006 showed that widening
`rerank_width` / `ec_spire.max_candidate_rows` from `25` to `200` did not
change recall; this packet measures the latency/storage cost of the remaining
nprobe tradeoff points.

## Scope

- Fresh PG18 release loads for 10k, 50k, and 100k staged real corpora.
- `ec_spire` with `turboquant` and `rabitq`, bits `4`, `k=10`, `queries_limit=100`.
- Fixed `rerank_width=25`.
- Nprobe sweep `24/96/192`.
- TQ runs used `ec_spire.pre_materialization_prune=on` and
  `ec_spire.max_candidate_rows=25`. RaBitQ runs used
  `ec_spire.max_candidate_rows=25`.

## Evidence

- Manifest: `artifacts/manifest.md`
- Suite config: `artifacts/task122-spire-latency-storage-width25.json`
- Suite results: `artifacts/suite/results.jsonl`
- Suite manifest: `artifacts/suite/suite-manifest.json`
- Audit log: `artifacts/suite-audit.log`
- Run log: `artifacts/suite-run.log`
- Backend/GUC check: `artifacts/guc-check.log`

The suite audit passed (`18` steps), and the final suite manifest records all
`18` steps as succeeded on a release backend.

## Results

Mean latency:

```text
scale  format      nprobe 24  nprobe 96  nprobe 192
10k    turboquant  2.21 ms    4.70 ms    4.90 ms
10k    rabitq      2.19 ms    4.62 ms    4.78 ms
50k    turboquant  4.49 ms    10.2 ms    17.3 ms
50k    rabitq      4.51 ms    9.99 ms    17.2 ms
100k   turboquant  6.49 ms    14.8 ms    25.7 ms
100k   rabitq      6.39 ms    14.8 ms    25.6 ms
```

Total storage and ec_spire index size:

```text
scale  format      total     ec_spire index
10k    turboquant  167.9 MiB 8.9 MiB
10k    rabitq      168.0 MiB 9.0 MiB
50k    turboquant  836.3 MiB 41.4 MiB
50k    rabitq      836.5 MiB 41.6 MiB
100k   turboquant  1.6 GiB   81.4 MiB
100k   rabitq      1.6 GiB   81.7 MiB
```

Combined with packet 006 recall:

- 50k reaches recall@10 `1.0000` at `nprobe=192,width=25`, costing about
  `17.3 ms`.
- 100k reaches recall@10 `0.9980` at `nprobe=192,width=25`, costing about
  `25.7 ms`.
- TQ and RaBitQ are latency-equivalent and storage-equivalent in this SPIRE
  shape.

## Interpretation

This narrows Task 122's next steps:

- There is no product evidence yet that TQ beats RaBitQ as the SPIRE
  pre-rerank representation at width `25`.
- The current measurable axis is nprobe quality/latency, not TQ candidate-width
  reduction.
- The next useful slice should inspect whether the engine actually has a
  distinct TQ candidate-reducer path before f32 rerank, or whether comparator
  work should move to a different access method where TQ can change the scoring
  or materialization surface.

This is still not a Task 122 closeout request. It is another checkpoint in the
seven-phase optimization exploration.
