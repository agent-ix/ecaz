# Task 121 Phase 2 Local Axis-Fix Prep Artifacts

- head_sha: `1575877c5e5f91e92865de6b0022c41e71d687f1`
- task_bucket: `reviews/task-121`
- packet: `reviews/task-121/010-phase2-local-axis-fix-prep`
- scope: dry-run-only preparation for the corrected Phase 2 local factorial
  benchmark matrix after reviewer feedback on packets 007/008/009
- timestamp: `2026-06-23T14:36:26Z`
- lane: `intel-local`
- fixture: staged local real corpora at 10k, 50k, and 100k
- storage format: `rabitq`
- rerank mode: default SPIRE pipeline exact-source rerank
- index/table isolation: planned isolated prefix/table/index per factorial cell
- AWS usage: none
- benchmark execution: not run; waiting on corrected-grid acceptance

## Configs

### `suite-phase2-local-factorial-axis-fix.json`

- SuiteConfig for the full corrected Task 121 Phase 2 local factorial grid.
- step count: `148`
- cells: `48`
- axes:
  - scale: `10k`, `50k`, `100k`
  - `boundary_replica_count`: `0`, `1`, `2`, `4`
  - `training_sample_rows`: `10000`, `50000`
  - `recursive_fanout`: `8`, `16`
  - `nlists`: fixed at `128`
  - `storage_format`: `rabitq`
  - nprobe sweep: `4,8,12,16,24,32,48,64,96`

### Slice configs

- `suite-phase2-local-10k-slice-axis-fix.json`
- `suite-phase2-local-50k-slice-axis-fix.json`
- `suite-phase2-local-100k-slice-axis-fix.json`

Each slice is the same corrected grid for one scale:

- step count: `50`
- cells: `16`
- step kinds: `raw=1`, `load=16`, `storage=16`, `recall=1`,
  `spire-pipeline=16`

## Audit

### Full grid

- command: `script -q -c "target/debug/ecaz bench suite audit --config reviews/task-121/010-phase2-local-axis-fix-prep/artifacts/suite-phase2-local-factorial-axis-fix.json" reviews/task-121/010-phase2-local-axis-fix-prep/artifacts/suite-phase2-local-factorial-axis-fix-audit.log`
- result: PASS
- key lines:
  - `[suite:task121-phase2-local-factorial-axis-fix] audit passed: 148 steps`
  - `COMMAND_EXIT_CODE="0"`

### Scale slices

- `suite-phase2-local-10k-slice-axis-fix-audit.log`: PASS,
  `audit passed: 50 steps`, `COMMAND_EXIT_CODE="0"`
- `suite-phase2-local-50k-slice-axis-fix-audit.log`: PASS,
  `audit passed: 50 steps`, `COMMAND_EXIT_CODE="0"`
- `suite-phase2-local-100k-slice-axis-fix-audit.log`: PASS,
  `audit passed: 50 steps`, `COMMAND_EXIT_CODE="0"`

## Dry Runs

### Full grid

- command: `script -q -c "target/debug/ecaz --database tqvector_bench_task121 --host /home/peter/.pgrx --port 28818 bench suite run --dry-run --config reviews/task-121/010-phase2-local-axis-fix-prep/artifacts/suite-phase2-local-factorial-axis-fix.json --manifest-output reviews/task-121/010-phase2-local-axis-fix-prep/artifacts/suite-phase2-local-factorial-axis-fix-manifest.json --results-output reviews/task-121/010-phase2-local-axis-fix-prep/artifacts/suite-phase2-local-factorial-axis-fix-results.jsonl --log-file reviews/task-121/010-phase2-local-axis-fix-prep/artifacts/suite-phase2-local-factorial-axis-fix.log" reviews/task-121/010-phase2-local-axis-fix-prep/artifacts/suite-phase2-local-factorial-axis-fix.script.log`
- result: PASS
- key lines:
  - `wrote reviews/task-121/010-phase2-local-axis-fix-prep/artifacts/suite-phase2-local-factorial-axis-fix-manifest.json`
  - `COMMAND_EXIT_CODE="0"`
- dry-run manifest:
  - `dry_run=true`
  - `config_sha256=4c5499af33a04c152d8ac3de787cdb9a44c015d7ca22785efe9d342904414693`
  - step count: `148`
  - step kinds: `raw=1`, `load=48`, `storage=48`, `recall=3`,
    `spire-pipeline=48`

### 10k slice

- command: `script -q -c "target/debug/ecaz --database tqvector_bench_task121 --host /home/peter/.pgrx --port 28818 bench suite run --dry-run --config reviews/task-121/010-phase2-local-axis-fix-prep/artifacts/suite-phase2-local-10k-slice-axis-fix.json --manifest-output reviews/task-121/010-phase2-local-axis-fix-prep/artifacts/suite-phase2-local-10k-slice-axis-fix-manifest.json --results-output reviews/task-121/010-phase2-local-axis-fix-prep/artifacts/suite-phase2-local-10k-slice-axis-fix-results.jsonl --log-file reviews/task-121/010-phase2-local-axis-fix-prep/artifacts/suite-phase2-local-10k-slice-axis-fix.log" reviews/task-121/010-phase2-local-axis-fix-prep/artifacts/suite-phase2-local-10k-slice-axis-fix.script.log`
- result: PASS
- dry-run manifest:
  - `dry_run=true`
  - `config_sha256=dde8c7703749057f2144cabd9084748f06155b36412c026274d45eec0a2b23ce`
  - step count: `50`
  - step kinds: `raw=1`, `load=16`, `storage=16`, `recall=1`,
    `spire-pipeline=16`

### 50k slice

- command: `script -q -c "target/debug/ecaz --database tqvector_bench_task121 --host /home/peter/.pgrx --port 28818 bench suite run --dry-run --config reviews/task-121/010-phase2-local-axis-fix-prep/artifacts/suite-phase2-local-50k-slice-axis-fix.json --manifest-output reviews/task-121/010-phase2-local-axis-fix-prep/artifacts/suite-phase2-local-50k-slice-axis-fix-manifest.json --results-output reviews/task-121/010-phase2-local-axis-fix-prep/artifacts/suite-phase2-local-50k-slice-axis-fix-results.jsonl --log-file reviews/task-121/010-phase2-local-axis-fix-prep/artifacts/suite-phase2-local-50k-slice-axis-fix.log" reviews/task-121/010-phase2-local-axis-fix-prep/artifacts/suite-phase2-local-50k-slice-axis-fix.script.log`
- result: PASS
- dry-run manifest:
  - `dry_run=true`
  - `config_sha256=890ead03c4a48e39a2ca41e621e0ac653f83b2dd26481e87f87d61197db89b4b`
  - step count: `50`
  - step kinds: `raw=1`, `load=16`, `storage=16`, `recall=1`,
    `spire-pipeline=16`

### 100k slice

- command: `script -q -c "target/debug/ecaz --database tqvector_bench_task121 --host /home/peter/.pgrx --port 28818 bench suite run --dry-run --config reviews/task-121/010-phase2-local-axis-fix-prep/artifacts/suite-phase2-local-100k-slice-axis-fix.json --manifest-output reviews/task-121/010-phase2-local-axis-fix-prep/artifacts/suite-phase2-local-100k-slice-axis-fix-manifest.json --results-output reviews/task-121/010-phase2-local-axis-fix-prep/artifacts/suite-phase2-local-100k-slice-axis-fix-results.jsonl --log-file reviews/task-121/010-phase2-local-axis-fix-prep/artifacts/suite-phase2-local-100k-slice-axis-fix.log" reviews/task-121/010-phase2-local-axis-fix-prep/artifacts/suite-phase2-local-100k-slice-axis-fix.script.log`
- result: PASS
- dry-run manifest:
  - `dry_run=true`
  - `config_sha256=9d498707c1c9d27031161a42e5188776e9fa9a317e7dab2d2a064ffbba571d40`
  - step count: `50`
  - step kinds: `raw=1`, `load=16`, `storage=16`, `recall=1`,
    `spire-pipeline=16`

## Representative Generated Commands

- 10k baseline fanout-8 load:
  - name: `load-10k_b0_tr10_f8`
  - prefix: `t121_s2_10k_b0_tr10_f8`
  - reloptions include `nlists=128`, `recursive_fanout=8`,
    `boundary_replica_count=0`, `training_sample_rows=10000`
- 10k fanout-16 replacement cell:
  - name: `load-10k_b0_tr10_f16`
  - prefix: `t121_s2_10k_b0_tr10_f16`
  - reloptions include `nlists=128`, `recursive_fanout=16`,
    `boundary_replica_count=0`, `training_sample_rows=10000`
- 100k high-coverage/cost cell:
  - name: `pipeline-100k_b4_tr50_f16`
  - prefix: `t121_s2_100k_b4_tr50_f16`
  - sweep: `4,8,12,16,24,32,48,64,96`
  - includes recall, query metrics, cost snapshot, local-store overlap,
    funnel JSONL, and stage-containment JSONL

## Staged Inputs

Verified by audit under `data/staged-current/`:

- `ec_real_10k_corpus.tsv`, `ec_real_10k_queries.tsv`, `ec_real_10k_manifest.json`
- `ec_real_50k_corpus.tsv`, `ec_real_50k_queries.tsv`, `ec_real_50k_manifest.json`
- `ec_real_100k_corpus.tsv`, `ec_real_100k_queries.tsv`, `ec_real_100k_manifest.json`

These are local input paths only; no corpus/query TSVs are committed in this
packet. Dry-runs reference planned `truth-cache-*.json` outputs, but no truth
cache files were generated or committed.
