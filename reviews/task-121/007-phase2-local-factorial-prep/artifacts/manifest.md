# Task 121 Phase 2 Local Factorial Prep Artifacts

- head_sha: `bc804159e0d53eb343ee3dde416304e84764c593`
- task_bucket: `reviews/task-121`
- packet: `reviews/task-121/007-phase2-local-factorial-prep`
- scope: dry-run-only preparation for the Phase 2 local factorial benchmark matrix
- timestamp: `2026-06-23T12:45:21Z`
- lane: `intel-local`
- fixture: staged local real corpora at 10k, 50k, and 100k
- storage format: `rabitq`
- rerank mode: default SPIRE pipeline exact-source rerank
- index/table isolation: planned isolated prefix/table/index per factorial cell
- AWS usage: none
- benchmark execution: not run; waiting on Phase 2 sign-off/explicit override

## Config

### `suite-phase2-local-factorial-dryrun.json`

- SuiteConfig for the full Task 121 Phase 2 local factorial grid.
- step count: `148`
- cells: `48`
- axes:
  - scale: `10k`, `50k`, `100k`
  - `boundary_replica_count`: `0`, `1`, `2`, `4`
  - `training_sample_rows`: `10000`, `50000`
  - `nlists`: `128`, `316`
  - `storage_format`: `rabitq`
  - nprobe sweep: `4,8,12,16,24,32,48,64,96`
- non-scope encoded in config doc:
  - PQ excluded
  - TurboQuant held for later compatibility/Pareto control, not route-factorial recovery

## Audit

### `suite-phase2-local-factorial-dryrun-audit.log`

- command: `script -q -c "target/debug/ecaz bench suite audit --config reviews/task-121/007-phase2-local-factorial-prep/artifacts/suite-phase2-local-factorial-dryrun.json" reviews/task-121/007-phase2-local-factorial-prep/artifacts/suite-phase2-local-factorial-dryrun-audit.log`

- result: PASS
- key lines:
  - `[suite:task121-phase2-local-factorial-dryrun] audit passed: 148 steps`
  - `COMMAND_EXIT_CODE="0"`

## Dry Run

### `suite-phase2-local-factorial-dryrun.script.log`

- command: `script -q -c "target/debug/ecaz --database tqvector_bench_task121 --host /home/peter/.pgrx --port 28818 bench suite run --dry-run --config reviews/task-121/007-phase2-local-factorial-prep/artifacts/suite-phase2-local-factorial-dryrun.json --manifest-output reviews/task-121/007-phase2-local-factorial-prep/artifacts/suite-phase2-local-factorial-dryrun-manifest.json --results-output reviews/task-121/007-phase2-local-factorial-prep/artifacts/suite-phase2-local-factorial-dryrun-results.jsonl --log-file reviews/task-121/007-phase2-local-factorial-prep/artifacts/suite-phase2-local-factorial-dryrun.log" reviews/task-121/007-phase2-local-factorial-prep/artifacts/suite-phase2-local-factorial-dryrun.script.log`
- result: PASS
- key lines:
  - `wrote reviews/task-121/007-phase2-local-factorial-prep/artifacts/suite-phase2-local-factorial-dryrun-manifest.json`
  - `COMMAND_EXIT_CODE="0"`

### `suite-phase2-local-factorial-dryrun-manifest.json`

- dry_run: `true`
- config_sha256: `4849b3b19555ffaf43fd93f055ef9d1afc3cc708fc5fe471955ae194fc816bbf`
- step count: `148`
- step kinds:
  - `raw`: `1`
  - `load`: `48`
  - `storage`: `48`
  - `recall`: `3`
  - `spire-pipeline`: `48`

Representative generated commands:

- load baseline 10k cell:
  - name: `load-10k_b0_tr10_n128`
  - prefix: `t121_s2_10k_b0_tr10_n128`
  - corpus: `data/staged-current/ec_real_10k_corpus.tsv`
  - storage format: `rabitq`
  - reloptions include `nlists=128`, `boundary_replica_count=0`, `training_sample_rows=10000`
- truth cache 10k:
  - name: `truth-cache-10k-q200-k10`
  - prefix: `t121_s2_10k_b0_tr10_n128`
  - queries limit: `200`
  - truth cache: `reviews/task-121/007-phase2-local-factorial-prep/artifacts/truth-cache-10k-q200-k10.json`
- high-coverage/cost 10k interaction cell:
  - name: `pipeline-10k_b4_tr50_n316`
  - prefix: `t121_s2_10k_b4_tr50_n316`
  - sweep: `4,8,12,16,24,32,48,64,96`
  - includes recall, query metrics, cost snapshot, local-store overlap, funnel JSONL, and stage-containment JSONL
- high-coverage/cost 100k interaction cell:
  - name: `pipeline-100k_b4_tr50_n316`
  - prefix: `t121_s2_100k_b4_tr50_n316`
  - sweep: `4,8,12,16,24,32,48,64,96`
  - includes recall, query metrics, cost snapshot, local-store overlap, funnel JSONL, and stage-containment JSONL

## Staged Inputs

Verified present under `data/staged-current/`:

- `ec_real_10k_corpus.tsv`, `ec_real_10k_queries.tsv`, `ec_real_10k_manifest.json`
- `ec_real_50k_corpus.tsv`, `ec_real_50k_queries.tsv`, `ec_real_50k_manifest.json`
- `ec_real_100k_corpus.tsv`, `ec_real_100k_queries.tsv`, `ec_real_100k_manifest.json`

These are local input paths only; no corpus/query TSVs are committed in this packet.
