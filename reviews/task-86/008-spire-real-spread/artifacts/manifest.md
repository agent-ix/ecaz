# Task 86 Packet 008 Artifact Manifest

- Packet: `reviews/task-86/008-spire-real-spread`
- Generated: `2026-06-07T18:45:33Z`
- Lane: local PG18 SPIRE TurboQuant real-corpus before/after benchmark
- Fixture: `data/task31_m5_dbpedia_staged` real10k / real50k / real100k DBPedia corpora
- Profile: `ec_spire`
- Storage format: `turboquant`
- Bits / seed: `4` / `42`
- Query limit: `200`
- Latency iterations: `1000`
- Surface: isolated one-index-per-table prefixes in two separate benchmark databases

## Source / Install Identity

- Baseline source commit: `eda36f088dfafc1c3c379de7f3e0cfac888fae06`
- Baseline worktree: `/Users/peter/dev/tqvector/.task-worktrees/task86-prelut`
- Baseline install log: `artifacts/lutoff/install-prelut-extension-rerun.log`
- Baseline installed library SHA256: `de57f9b8c741152d1022cdd88eb16ce2db3cee9bd1ecc1f57e534d290181e628`
- Current source commit used for extension code: `c200632f5835b3a0cd08938f3e9cdff5b836a8f9`
- Packet head after reviewer feedback: `0d93ef0de47bf10a2aa455204e4e0b97ce89be54`
- Current install log: `artifacts/luton/install-luton-extension-rerun.log`
- Current installed library SHA256: `86d7e841cfe6e31e8347c961172b2b3523f63a7a97fafd32918ac15e056e465c`

## Suite Configs

### `suite-lutoff.json`

- Command:
  `/Users/peter/dev/tqvector/target/debug/ecaz bench suite audit --config reviews/task-86/008-spire-real-spread/suite-lutoff.json > reviews/task-86/008-spire-real-spread/artifacts/audit-lutoff.log 2>&1`
- Audit result: `audit passed: 16 steps`
- Run command:
  `/Users/peter/dev/tqvector/target/debug/ecaz --database task86_spire_lutoff --host /Users/peter/.pgrx --port 28818 bench suite run --config reviews/task-86/008-spire-real-spread/suite-lutoff.json --manifest-output reviews/task-86/008-spire-real-spread/artifacts/lutoff/suite-manifest-rerun.json --results-output reviews/task-86/008-spire-real-spread/artifacts/lutoff/results-rerun.jsonl --log-file reviews/task-86/008-spire-real-spread/artifacts/lutoff/suite-run-rerun.log`
- Suite report: `artifacts/lutoff/suite-report.md`
- Normalized result rows: `artifacts/lutoff/results-report.jsonl`

### `suite-luton.json`

- Command:
  `/Users/peter/dev/tqvector/target/debug/ecaz bench suite audit --config reviews/task-86/008-spire-real-spread/suite-luton.json > reviews/task-86/008-spire-real-spread/artifacts/audit-luton.log 2>&1`
- Audit result: `audit passed: 16 steps`
- Run command:
  `/Users/peter/dev/tqvector/target/debug/ecaz --database task86_spire_luton --host /Users/peter/.pgrx --port 28818 bench suite run --config reviews/task-86/008-spire-real-spread/suite-luton.json --manifest-output reviews/task-86/008-spire-real-spread/artifacts/luton/suite-manifest-rerun.json --results-output reviews/task-86/008-spire-real-spread/artifacts/luton/results-rerun.jsonl --log-file reviews/task-86/008-spire-real-spread/artifacts/luton/suite-run-rerun.log`
- Suite report: `artifacts/luton/suite-report.md`
- Normalized result rows: `artifacts/luton/results-report.jsonl`

## Key Result Lines

- Recall: identical at all nine sweep points.
- SQL mean latency:
  - real10k: `3.44 -> 3.30 ms`, `8.02 -> 7.69 ms`, `10.2 -> 9.74 ms`
  - real50k: `12.3 -> 12.0 ms`, `33.9 -> 32.9 ms`, `48.0 -> 46.1 ms`
  - real100k: `25.3 -> 24.5 ms`, `74.1 -> 71.7 ms`, `95.3 -> 92.3 ms`
- Pipeline p50:
  - real10k: `3.549 -> 3.406 ms`, `8.089 -> 7.675 ms`, `10.283 -> 9.711 ms`
  - real50k: `12.660 -> 11.938 ms`, `33.779 -> 32.299 ms`, `48.192 -> 46.042 ms`
  - real100k: `25.646 -> 24.670 ms`, `74.584 -> 72.274 ms`, `95.084 -> 92.184 ms`
- SQL p95/p99 and pipeline p95/p99 are reported in
  `artifacts/benchmark-delta.md` and sourced from
  `artifacts/lutoff/results-report.jsonl` and
  `artifacts/luton/results-report.jsonl`.
- SPIRE index storage unchanged:
  - real10k: `8.2 MiB`, `857.7 B/row`
  - real50k: `39.8 MiB`, `834.1 B/row`
  - real100k: `79.5 MiB`, `833.9 B/row`

## Reports

- Delta report: `artifacts/benchmark-delta.md`
- Requirements re-audit: `artifacts/requirements-audit.md`
