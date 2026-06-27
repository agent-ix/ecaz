# Legacy 0x2A Attribution Audit

Packet: `reviews/task-111h/033-legacy-0x2a-attribution`

Current head audited: `28a5cf08fce3d2f233e296111cbbf597e84775a3`

Legacy benchmark source packet:
`reviews/task-111g/005-direct-sidecar-rerank-tids`

Legacy benchmark implementation commit:
`a7cdb86fe021fa11db0ea00ac07c47c8896d7f1a`

## Verdict

Task 111h's legacy `0x2A` direct-TID sidecar baseline is already benchmarked in
`reviews/task-111g/005-direct-sidecar-rerank-tids`. The completed suite is
packet-local there and should be treated as the durable legacy baseline:

- suite status:
  `reviews/task-111g/005-direct-sidecar-rerank-tids/artifacts/suite-status.log`
- suite config:
  `reviews/task-111g/005-direct-sidecar-rerank-tids/artifacts/sidecar-index-direct-tids/suite-config.json`
- suite manifest:
  `reviews/task-111g/005-direct-sidecar-rerank-tids/artifacts/sidecar-index-direct-tids/suite-manifest.json`
- normalized results:
  `reviews/task-111g/005-direct-sidecar-rerank-tids/artifacts/sidecar-index-direct-tids/results.jsonl`
- latency, recall, storage, and load logs:
  `reviews/task-111g/005-direct-sidecar-rerank-tids/artifacts/sidecar-index-direct-tids/{latency,recall,storage,load}-*.log`

The current 111h/v7 branch cannot rerun `0x2A` directly: current builds write the
packed rerank group/segment layout (`0x2B`/`0x2C`), and current metadata decode
rejects v4, the legacy `0x2A` metadata version. A rerun of the legacy path would
need to check out the legacy implementation commit or its packet state, not run
against current v7 HEAD.

## Current-Head Format Audit

Current v7 writer/read policy:

- `src/am/ec_ivf/page.rs` declares `EC_IVF_INDEX_FORMAT_VERSION = 7`.
- The current metadata comment states v7 is the only supported on-disk format,
  v7 points `rerank_sidecar_head` at the `0x2B` packed group-header chain, and
  v4's `0x2A` heap-TID sidecar is legacy only.
- `tests/on_disk_fixtures.rs` rejects `ivf_metadata_v4.hex` with
  `unsupported ec_ivf metadata format version: 4`.
- `docs/on-disk-format.md` documents `0x2A` as a legacy v4 rerank sidecar block
  and says v7 readers reject v4 metadata rather than reading it.

Current build/scan path:

- `src/am/ec_ivf/build.rs` calls `build_rerank_group_chain` for
  `rerank_placement = 'index'`, which writes packed rerank group headers and
  segments.
- Current postings carry direct pointers to packed group headers.
- `src/am/ec_ivf/scan.rs` chooses `rerank_probe_candidates_index_side` when
  `rerank_sidecar_head` is valid; that helper loads packed groups by header TID
  or via the full packed-group chain fallback.
- `src/am/ec_ivf/page.rs` still contains `IvfRerankSidecarBlockTuple` and
  `read_ivf_rerank_sidecar_block` utilities for the old tuple shape, but current
  source search shows no build/insert writer caller for
  `append_ivf_rerank_sidecar_block_to_new_block` or
  `IvfRerankSidecarBlockTuple::new` outside page tests.

## Legacy Direct-TID 0x2A Benchmark Evidence

The legacy direct-TID suite was driven by `ecaz bench suite` and completed:

```text
[suite:ivf-attr-sidecar-index-placement] completed=24 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0
```

Source:
`reviews/task-111g/005-direct-sidecar-rerank-tids/artifacts/suite-status.log`

The packet manifest records:

- head SHA: `a7cdb86fe021fa11db0ea00ac07c47c8896d7f1a`
- run surface: isolated one-index-per-table prefixes
  `attr_idx_{f16,rq4}_{10k,50k,100k}`
- storage format: `storage_format=coarse_rerank`
- placement: `rerank_placement=index`
- width: `rerank_width=64`
- formats: `f16`, `rabitq4`
- results: `sidecar-index-direct-tids/results.jsonl`

### 100k Headline Rows

All rows below are warm-cache local `post_recall_warm`, `iterations=200`,
`concurrency=1`, from the 111g/005 normalized results.

| Legacy `0x2A` direct-TID cell | nprobe 8 | nprobe 64 | nprobe 200 |
| --- | ---: | ---: | ---: |
| f16 recall@10 | 0.7670 | 0.9640 | 0.9975 |
| f16 p50 latency | 2.99 ms | 6.02 ms | 13.0 ms |
| rabitq4 recall@10 | 0.7465 | 0.9165 | 0.9420 |
| rabitq4 p50 latency | 2.79 ms | 5.72 ms | 11.9 ms |

100k storage rows:

| Legacy `0x2A` direct-TID cell | ec_ivf index size | per-row index bytes |
| --- | ---: | ---: |
| f16 | 416.6 MiB | 4368.4 B |
| rabitq4 | 103.6 MiB | 1086.8 B |

### Interpretation

The old directory-only ADR-079 result, where f16 was around `150 ms` at 100k,
was a real measurement of the old per-query directory/materialization path, not
an inherent f16 scorer/storage result.

The direct-TID `0x2A` packet supersedes that old interpretation:

| cell | nprobe 8 | nprobe 64 | nprobe 200 |
| --- | ---: | ---: | ---: |
| f16 index 100k, old ADR-079 packet | 146.8 ms | 150.2 ms | 159.2 ms |
| f16 index 100k, direct-TID `0x2A` packet | 2.99 ms | 6.02 ms | 13.0 ms |
| rabitq4 index 100k, old ADR-079 packet | 7.67 ms | 9.60 ms | 16.0 ms |
| rabitq4 index 100k, direct-TID `0x2A` packet | 2.79 ms | 5.72 ms | 11.9 ms |

The remaining f16 concern in this legacy baseline is storage footprint, not a
150 ms query-latency failure: f16 index-side was `416.6 MiB` at 100k in the
legacy direct-TID suite.

## Relationship To Current v7 Packed Groups

The v7 100k matched-recall packet reports:

- source/f32 at recall target 0.95: `w32 np64`, recall `0.9625`,
  p50 `6.23 ms`, index `24.6 MiB`
- index/f16 at recall target 0.95: `w32 np64`, recall `0.9620`,
  p50 `6.51 ms`, index `342.0 MiB`
- index/rabitq4 does not reach 0.95: best cited row `w64 np200`,
  recall `0.9380`, p50 `15.3 ms`, index `110.2 MiB`

Source:
`reviews/task-111h/029-cross-scale-matched-recall-v7/artifacts/cross-scale-matched-recall-v7.md`

This means the legacy `0x2A` direct-TID evidence is useful attribution, but it
does not supersede the current v7 decision frontier:

- `0x2A` proved direct physical sidecar TIDs fixed the old directory-scan bug.
- v7 packed groups are the current layout under decision.
- The current warm-cache frontier still favors source/f32; index/f16 remains
  recall-neutral but storage-heavy, while rabitq4 remains below the 0.95 target
  at 100k.

## Closeout Impact

This packet closes the Task 111h checklist row:

```text
Benchmark the existing 0x2A direct-TID sidecar path as a legacy index-side baseline before replacing or superseding it.
```

It does not close table-owned storage, RaBitQ slab cleanup/evidence,
cold/remote evidence, or the final promote/iterate/abandon decision.
