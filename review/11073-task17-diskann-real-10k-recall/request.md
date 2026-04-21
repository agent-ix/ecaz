# Review Request: pg18 real-10k DiskANN Recall@10 artifact via `ecaz`

Branch: `adr034-diskann-rebased`
Author: coder-2
Target:

- `review/11073-task17-diskann-real-10k-recall/artifacts/load-prefix-mismatch.log`
- `review/11073-task17-diskann-real-10k-recall/artifacts/load.log`
- `review/11073-task17-diskann-real-10k-recall/artifacts/recall.log`
- `review/11073-task17-diskann-real-10k-recall/artifacts/manifest.md`

## What this packet is

This is the actual task-17 measurement artifact packet for the canonical pg18
`ecaz` path:

```text
ecaz corpus fetch
ecaz corpus prepare
ecaz corpus load --profile ec_diskann ...
ecaz bench recall --profile ec_diskann ...
```

The result captured here is the single-point real-corpus gate run requested for
DiskANN:

- fixture prefix: `ec_hnsw_real_10k`
- profile: `ec_diskann`
- reloptions: `graph_degree=32`, `build_list_size=100`, `alpha=1.2`
- sweep point: `list_size=128`
- metric: Recall@10 on 200 real queries vs 10,000 real corpus rows

## Result

From `artifacts/recall.log`:

```text
┌───────────┬──────────┬────────┬─────────────┐
│ list_size ┆ recall@k ┆ ndcg@k ┆ mean q-time │
╞═══════════╪══════════╪════════╪═════════════╡
│ 128       ┆ 0.0075   ┆ 0.4833 ┆ 38.23 ms    │
└───────────┴──────────┴────────┴─────────────┘
```

The same log also captures:

```text
[recall] ground truth in 4.52s
```

## Artifact notes

- `artifacts/load-prefix-mismatch.log` records the first failed load attempt
  when I tried to rename the prepared fixture to `ec_diskann_real_10k`. The
  manifest correctly rejected that: the prepared fixture must be loaded under
  its manifest prefix `ec_hnsw_real_10k`.
- `artifacts/load.log` records the successful canonical load using that
  manifest-matching prefix and the `ec_diskann` index profile.
- `artifacts/manifest.md` is the packet-local source of truth for commands,
  timestamps, surface metadata, and cited result lines.

## Why this packet matters

This packet closes the real-corpus task-17 path with the actual `ecaz` surface
rather than ad hoc SQL or env-var wrappers. The generic CLI blockers uncovered
along the way were split into their own standalone packets:

- `11075` `ecaz corpus load` ensures `CREATE EXTENSION ecaz`
- `11076` ecaz KNN SQL uses the right operator/type resolution path
- `11077` measurement commands force the ordered ANN path instead of seqscan
  fallback

With those in place, this packet is the direct pg18 DiskANN artifact, not a
tooling placeholder.

## Test evidence

```text
$ cargo test -p ecaz-cli --quiet

test result: ok. 218 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

Also passed on `pg18` for the final code checkpoint behind this artifact:

- `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`

## Follow-ups intentionally not in this packet

- Sweep expansion beyond `list_size=128`. This packet captures the requested
  single-point gate, not a Pareto sweep.
- Interpretation of whether `Recall@10 = 0.0075` is acceptable. This packet
  only captures the measured result and the exact artifact trail.
