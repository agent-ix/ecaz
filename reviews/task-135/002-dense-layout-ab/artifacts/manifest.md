# Task 135 packet 002 — dense-layout A/B (row vs dense_posting_blocks=1): manifest

- Head SHA (dylib + runner): `8f7bce3cc` (branch `task-136-rank1-scorer`; the
  Task 136 scorer GUC defaults off and is inert — both cells run the LUT
  scorer). In-suite verification: `precheck-build-sha.log` records
  `ecaz_build_git_sha()` = `8f7bce3cc7bed148b97939d519665dcf922f1878`,
  captured 2026-07-02 09:02 local, shared_buffers 128MB.
- Task bucket / packet: `reviews/task-135/002-dense-layout-ab/`
- Host: Apple M5 Pro (m5-local), PG18 socket `/Users/peter/.pgrx` port 28818,
  db `tqvector_bench`. No dylib installs during the run.
- Lane / fixture: IVF `ec_ivf`, storage_format `turboquant` (no-QJL 4-bit,
  dim 1536, bits 4, seed 42), dbpedia staged corpora
  `data/staged-current/ec_real_{10k,50k,100k}_*`. nprobe [32], k 10, warm
  cache, concurrency 1.
- A/B axis: **index reloption** `dense_posting_blocks=1` (shipped, gated
  Task 111 dense posting-block layout) vs default row layout. Scan-side GUCs
  at defaults (`dense_posting_coalescing=on`, `dense_posting_typed_views=on` —
  the landed Task 111a Approach-A path). One lever; same session, same binary;
  per scale both cells loaded and measured back-to-back.
- Surfaces: isolated one-index-per-table, fresh loads
  (`task135ab_{row,dense}_real{scale}` prefixes).
- Runner command:

  ```sh
  target/release/ecaz --database tqvector_bench --host /Users/peter/.pgrx --port 28818 \
    bench suite run --config reviews/task-135/002-dense-layout-ab/task135-dense-layout-ab-suite.json \
    --artifact-dir reviews/task-135/002-dense-layout-ab/artifacts
  ```

- Bespoke config reason: adds the `dense_posting_blocks` reloption axis and
  fixed nprobe=32 stage-counter point that the canonical lane config does not
  carry; shape copied from the task-133/136 A/B configs.
- Completed 2026-07-02 ~09:45 local, exit 0, 25/25 steps
  (`suite-manifest.json`, `results.jsonl`, `suite-run.log`).
- `[loader] warning: manifest verification failed ... prefix` lines in
  `suite-run.log` are the expected `allow_manifest_mismatch` prefix warnings
  (staged manifests record the `ec_real_*` prefixes), same as prior packets.

## Key result lines (cited by request.md)

### Recall@10 / NDCG@10 (nprobe=32) — byte-identical

| scale | row | dense |
|---|---|---|
| 10k (64q) | 0.9734 / 0.9998 | 0.9734 / 0.9998 |
| 50k (48q) | 0.9521 / 0.9995 | 0.9521 / 0.9995 |
| 100k (32q) | 0.8969 / 0.9957 | 0.8969 / 0.9957 |

### Latency (ms, warm, concurrency 1)

| scale | row mean / p50 / p95 | dense mean / p50 / p95 | mean delta |
|---|---|---|---|
| 10k | 0.98 / 0.93 / 1.17 | 0.90 / 0.87 / 1.02 | −8.2% |
| 50k | 1.81 / 1.75 / 2.01 | 1.73 / 1.70 / 1.89 | −4.4% |
| 100k | 2.82 / 2.72 / 3.44 | 2.74 / 2.66 / 3.30 | −2.8% |

No p95/p99 regression at any scale.

### Stage counters (per-sweep elapsed_ms)

| scale | stage | row | dense | delta |
|---|---|---|---|---|
| 10k | posting_visit | 40.333 | 36.962 | −8.4% |
| 10k | visit − flush | 13.787 | 10.121 | **−26.6%** |
| 50k | posting_visit | 63.069 | 59.215 | −6.1% |
| 50k | visit − flush | 19.821 | 14.050 | **−29.1%** |
| 100k | posting_visit | 68.701 | 65.032 | −5.3% |
| 100k | visit − flush | 25.105 | 19.099 | **−23.9%** |

Sub-stage split (Task 135 packet 001 timer), 100k per-sweep:
parse+push (page_decode − flush) 19.983 → 13.884 (−30.5%); page/buffer access
(visit − page_decode) 5.122 → 5.215 (flat — fresh row loads at 128MB
shared_buffers show far lower page-access cost than the older tables profiled
in packet 001; the dense win here is the entry parse/copy count, and the A/B
is same-session so the comparison stands).

### Scorer-side counter-effect (caveat)

Dense raises flush count and narrows widths: 100k flushes 1311 → 1781
(width histogram lt8/8-15/16-31/ge32: row 2/0/2/1307 → dense 22/38/33/1688),
scorer_batch 39.941 → 42.827 ms (+7.2%). The dense-coalesced scratch drains at
row/list boundaries instead of accumulating to the 256-posting target, giving
back ~40% of the parse+push win at 100k. Follow-up lever noted in request.md.

### Storage (turboquant index)

| scale | row | dense | delta |
|---|---|---|---|
| 10k | 9.8 MiB (1028.1 B/row) | 9.0 MiB (939.6 B/row) | −8.2% |
| 50k | 46.0 MiB (964.7 B/row) | 41.6 MiB (872.0 B/row) | −9.6% |
| 100k | 90.4 MiB (948.2 B/row) | 81.7 MiB (856.5 B/row) | −9.6% |

### Build timing (100k, from load logs)

Near-identical: stage_postings_us 25,901 (row) vs 22,760 (dense);
train/assign stages equal within noise.

## Not committed (regenerable / banned)

- `truth-cache/` (gitignored).
