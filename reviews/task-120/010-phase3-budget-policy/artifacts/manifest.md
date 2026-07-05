# Task 120 Packet 010 Artifact Manifest

- benchmark head SHA: `c7c3f661293bd6f5a0b96000829c4fe2d230ae1a`
- packet write-up base SHA: `73cfe3ead` (packet 009 hygiene-only commit after
  the run; no source or benchmark-affecting files changed)
- task bucket: `reviews/task-120/`
- packet path: `reviews/task-120/010-phase3-budget-policy/`
- lane: local PG18 Intel host
- database/socket: `tqvector_bench_task120` on `/home/peter/.pgrx`, port `28818`
- fixture: staged real corpus at `data/staged-current/ec_real_{10k,50k,100k}_corpus.tsv`
- access method / quantizer: `ec_spire` recursive RabitQ f8/b64/L2
- surfaces: isolated one-index-per-table prefixes reused from packet 008
  (`task120_phase2_real{10k,50k,100k}_spire_rabitq_f8_b64_l2`), not shared-table
- rerank modes: default `rerank_width=25`; explicit `rerank_width=25,100,500`
  with `max_candidate_rows=0`; cap-only `max_candidate_rows=10000,25000`
- remote/distributed: false; this is local leaf policy evidence only
- run timestamp: `2026-06-21 12:32:29 -0700` to `2026-06-21 12:55:08 -0700`
- corpus data: TSV corpus/query/truth inputs were not committed; the suite reads
  staged local data and records the file paths in command lines

## Host And Runner

- `precheck-host.log`
  - command: recorded by `suite-status.log` as `dev sql --pg 18 ...`
  - key result: PostgreSQL `18.3`, `ecaz_build_profile=release`,
    `ec_spire.nprobe=-1`, `ec_spire.rerank_width=-1`,
    `ec_spire.max_candidate_rows=-1`, `ec_spire.max_routed_candidate_rows=0`,
    leaf block pruning caps `0`
- `suite.json`
  - checked-in task-local `SuiteConfig`
  - config SHA256: `d2c745799640084ba3000dee8cbf35ff3317fcbfa2a9ea7cb337d32b08c163ae`
  - steps: 1 host precheck, 3 storage probes, 18 pipeline policy variants
  - query limit: 200 queries per scale
  - sweep: `nprobe=8,16,24,32`
  - recall source: `--truth-corpus-file data/staged-current/ec_real_*_corpus.tsv`
- `suite-audit.log`
  - command: `target/debug/ecaz bench suite audit --config reviews/task-120/010-phase3-budget-policy/artifacts/suite.json --database tqvector_bench_task120 --host /home/peter/.pgrx --port 28818 --log-file reviews/task-120/010-phase3-budget-policy/artifacts/suite-audit.log`
  - key result: `audit passed: 22 steps`
- `suite-dry-run.log`, `suite-manifest.dry-run.json`
  - command: `target/debug/ecaz bench suite run --dry-run --config reviews/task-120/010-phase3-budget-policy/artifacts/suite.json --database tqvector_bench_task120 --host /home/peter/.pgrx --port 28818 --manifest-output reviews/task-120/010-phase3-budget-policy/artifacts/suite-manifest.dry-run.json --log-file reviews/task-120/010-phase3-budget-policy/artifacts/suite-dry-run.log`
- `suite-run.log`, `suite-manifest.json`, `suite-results.jsonl`
  - command: `target/debug/ecaz bench suite run --config reviews/task-120/010-phase3-budget-policy/artifacts/suite.json --database tqvector_bench_task120 --host /home/peter/.pgrx --port 28818 --manifest-output reviews/task-120/010-phase3-budget-policy/artifacts/suite-manifest.json --results-output reviews/task-120/010-phase3-budget-policy/artifacts/suite-results.jsonl --log-file reviews/task-120/010-phase3-budget-policy/artifacts/suite-run.log`
- `suite-status.log`
  - command: `target/debug/ecaz bench suite status --manifest reviews/task-120/010-phase3-budget-policy/artifacts/suite-manifest.json --log-file reviews/task-120/010-phase3-budget-policy/artifacts/suite-status.log`
  - key result: `completed=22 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0`
- `suite-report.md`, `suite-report-results.jsonl`
  - command: `target/debug/ecaz bench suite report --manifest reviews/task-120/010-phase3-budget-policy/artifacts/suite-manifest.json --results-output reviews/task-120/010-phase3-budget-policy/artifacts/suite-report-results.jsonl --log-file reviews/task-120/010-phase3-budget-policy/artifacts/suite-report.md`
- `phase3-budget-policy-summary.txt`
  - compact packet-local source for the storage and nprobe-32 decision tables

## Per-Scale Artifacts

For each scale, the packet includes:

- `storage-{10k,50k,100k}-recursive-rabitq.log`
- `pipeline-{10k,50k,100k}-default.log`
- `pipeline-{10k,50k,100k}-cap10k.log`
- `pipeline-{10k,50k,100k}-cap25k.log`
- `pipeline-{10k,50k,100k}-cap0-w25.log`
- `pipeline-{10k,50k,100k}-cap0-w100.log`
- `pipeline-{10k,50k,100k}-cap0-w500.log`

No raw per-query pipeline JSONL families are committed in this packet. The
committed JSONL files are the suite runner's structured `suite-results.jsonl`
and `suite-report-results.jsonl`.

## Storage

| Scale | Rows | Total | Total bytes | ec_spire index | Index bytes |
| --- | ---: | ---: | ---: | ---: | ---: |
| 10k | 10,000 | 168.8 MiB | 176,999,629 | 9.7 MiB | 10,171,187 |
| 50k | 50,000 | 837.0 MiB | 877,658,112 | 42.1 MiB | 44,145,050 |
| 100k | 100,000 | 1.6 GiB | 1,717,986,918 | 82.5 MiB | 86,507,520 |

## Nprobe 32 Policy Result

| Scale | Variant | recall@10 | p50 | p95 | p99 | candidate_sum | object_bytes_sum | heap_rerank_sum |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 10k | default | 0.9965 | 8.668 ms | 9.756 ms | 10.717 ms | 520,143 | 442,719,420 | 5,000 |
| 10k | cap0-w100 | 0.9965 | 10.967 ms | 12.395 ms | 13.711 ms | 520,143 | 442,719,420 | 20,000 |
| 10k | cap0-w500 | 0.9965 | 24.260 ms | 26.335 ms | 29.583 ms | 520,143 | 442,719,420 | 100,000 |
| 50k | default | 0.9725 | 14.869 ms | 17.703 ms | 19.470 ms | 2,326,779 | 1,960,850,458 | 5,000 |
| 50k | cap0-w100 | 0.9725 | 17.272 ms | 19.663 ms | 21.238 ms | 2,326,779 | 1,960,850,458 | 20,000 |
| 50k | cap0-w500 | 0.9725 | 31.612 ms | 34.350 ms | 35.298 ms | 2,326,779 | 1,960,850,458 | 100,000 |
| 100k | default | 0.9310 | 25.396 ms | 27.574 ms | 30.848 ms | 5,165,224 | 4,344,876,152 | 5,000 |
| 100k | cap0-w100 | 0.9310 | 28.893 ms | 40.828 ms | 45.268 ms | 5,165,224 | 4,344,876,152 | 20,000 |
| 100k | cap0-w500 | 0.9310 | 44.393 ms | 62.181 ms | 66.186 ms | 5,165,224 | 4,344,876,152 | 100,000 |

## Decision

- Cap-only variants (`max_candidate_rows=10000` and `25000`) are recall-neutral
  under the default `rerank_width=25`; the heap rerank count remains 5,000 rows
  total for 200 queries.
- Explicit width variants (`rerank_width=100` and `500`) raise exact heap
  rerank volume to 20,000 and 100,000 rows total, but recall does not improve at
  any measured scale or nprobe.
- Candidate/read volume at nprobe 32 is already large (443 MB at 10k, 1.96 GB
  at 50k, 4.34 GB at 100k over 200 queries) and is not the axis these width
  settings improve.
- Go/no-go: do not promote a wider local leaf rerank width or candidate-cap
  policy as a product default from Phase 3. Keep these knobs diagnostic and move
  Task 120 Phase 4 to route/topology refinement.

## Guardrails

- This packet is not Task 120 closeout. Phases 4-6 and AWS/distributed evidence
  remain open.
- This packet does not claim a code change, storage improvement, or merge-ready
  product default.
- The suite used `ecaz bench suite`; no ad-hoc sweeper was added.
