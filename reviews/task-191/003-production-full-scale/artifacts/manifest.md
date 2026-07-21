# Task 191 full-scale artifact manifest

- Task bucket: `reviews/task-191/`
- Packet: `reviews/task-191/003-production-full-scale/`
- Suite runner HEAD: `bbea9dbd384457174af169285b7654f6c800056b`
- Extension implementation SHA: `7883cfcf8ca9769da72c3cf22856faa5537eb17f`
- Extension SHA-256:
  `6701d22880071a280668c84d4e2b43e4ebf997037a15a9032fe18879e997cd82`
- CLI SHA-256:
  `204297e7ec7c62b4adc81f80e7153900d96a0d4701aa1ae50fc5358a70cb285c`
- Timestamp: 2026-07-20 PDT
- Host/lane: Intel local, PG18 release, three loopback PostgreSQL owners
- Fixtures: staged `ec_real_10k`, `ec_real_50k`, `ec_real_100k`
- Quant/index: physical `ec_distann`, stored neighbor code `rabitq`
- Rerank mode: physical payload materialization
- Arms: feature-only eager control `0`; production lazy window `10`
- Isolation: one shared byte-identical physical generation per scale; A/B arms
  differ only in materialization override and have identical seed-ID digests

## Suite artifacts

### `full-scale-suite.json`

- Checked-in `ecaz bench suite` configuration.
- Required matrix: 10k/50k/100k, 200 queries / 2,000 top-10 trials, 50 warm
  latency samples after 10 warmups, concurrency one, eager versus lazy10.

### `suite-audit.log`

- Command: `ecaz bench suite audit --config <full-scale-suite.json>`
- Result: audit passed, three steps.

### `suite-dry-run.log`

- Command: `ecaz bench suite run --config <full-scale-suite.json> --dry-run`
- Result: expands all three scales with explicit eager `0` and lazy `10` arms.

### `full-run/suite-manifest.json`

- Command: `ecaz bench suite run --config <full-scale-suite.json>`
- Result: three succeeded steps; failed/skipped/missing/stale all zero.
- Clean runner descriptor:
  `bbea9dbd384457174af169285b7654f6c800056b`.

### `full-run/results.jsonl`

- Normalized source of truth for recall/Wilson intervals, latency distribution,
  stage/work/bytes, storage, construction, topology, engagement, query
  separation, and installed release provenance.

### `full-run/production-ab-{10k,50k,100k}/distann-multinode-summary.log`

- Compact accepted summaries for each scale.
- Key results:
  - recall eager/lazy: 10k `0.9990/0.9990`, 50k `0.9685/0.9685`,
    100k `0.9625/0.9625`;
  - mean latency eager/lazy: 10k `34.00/21.70 ms`, 50k
    `36.90/22.70 ms`, 100k `39.00/23.70 ms`;
  - p95 eager/lazy: 10k `39.70/25.10 ms`, 50k `44.20/26.20 ms`,
    100k `49.20/27.20 ms`;
  - remote candidates per scan eager/lazy: 10k `23.68/6.58`, 50k
    `26.36/6.72`, 100k `26.84/6.64`;
  - payload bytes per scan eager/lazy: 10k `437606.4/121598.4`, 50k
    `487132.8/124185.6`, 100k `496003.2/122707.2`;
  - duplicate remote candidate requests: zero for every arm and scale;
  - topology and remote engagement gates: pass at every scale;
  - storage and construction: identical between A/B arms at each scale.

### `suite-status.log`

- Command: `ecaz bench suite status --manifest <suite-manifest.json>`
- Result: completed 3; failed/skipped/dry-run/missing/stale all zero.

### `suite-report.md`

- Command: `ecaz bench suite report --manifest <suite-manifest.json>`
- Result: generated normalized Markdown report across all three accepted steps.

## Retention note

Raw node logs and per-arm recall/latency child logs were pruned after compact
summaries and normalized results were produced. Corpus/query/truth data are not
committed; the suite records staged prefixes and SHA-256 provenance instead.
