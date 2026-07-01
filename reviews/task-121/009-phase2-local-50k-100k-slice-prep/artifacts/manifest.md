# Task 121 Phase 2 Local 50k/100k Slice Prep Artifacts

- head_sha: `8f9786528bd3602d950b346f88163a87410baaa1`
- task_bucket: `reviews/task-121`
- packet: `reviews/task-121/009-phase2-local-50k-100k-slice-prep`
- scope: dry-run-only preparation for the Phase 2 local 50k and 100k factorial benchmark slices
- timestamp: `2026-06-23T12:52:42Z`
- lane: `intel-local`
- fixture: staged local real corpora at 50k and 100k
- storage format: `rabitq`
- rerank mode: default SPIRE pipeline exact-source rerank
- index/table isolation: planned isolated prefix/table/index per factorial cell
- AWS usage: none
- benchmark execution: not run; waiting on Phase 2 sign-off/explicit override

## 50k Config

### `suite-phase2-local-50k-slice-dryrun.json`

- derived from: `reviews/task-121/007-phase2-local-factorial-prep/artifacts/suite-phase2-local-factorial-dryrun.json`
- scale: `50k`
- step count: `50`
- cells: `16`
- axes:
  - `boundary_replica_count`: `0`, `1`, `2`, `4`
  - `training_sample_rows`: `10000`, `50000`
  - `nlists`: `128`, `316`
  - `storage_format`: `rabitq`
  - nprobe sweep: `4,8,12,16,24,32,48,64,96`
- non-scope:
  - PQ excluded
  - TurboQuant held for later compatibility/Pareto control

### `suite-phase2-local-50k-slice-dryrun-audit.log`

- command: `script -q -c "target/debug/ecaz bench suite audit --config reviews/task-121/009-phase2-local-50k-100k-slice-prep/artifacts/suite-phase2-local-50k-slice-dryrun.json" reviews/task-121/009-phase2-local-50k-100k-slice-prep/artifacts/suite-phase2-local-50k-slice-dryrun-audit.log`
- result: PASS
- key lines:
  - `[suite:task121-phase2-local-50k-slice-dryrun] audit passed: 50 steps`
  - `COMMAND_EXIT_CODE="0"`

### `suite-phase2-local-50k-slice-dryrun.script.log`

- command: `script -q -c "target/debug/ecaz --database tqvector_bench_task121 --host /home/peter/.pgrx --port 28818 bench suite run --dry-run --config reviews/task-121/009-phase2-local-50k-100k-slice-prep/artifacts/suite-phase2-local-50k-slice-dryrun.json --manifest-output reviews/task-121/009-phase2-local-50k-100k-slice-prep/artifacts/suite-phase2-local-50k-slice-dryrun-manifest.json --results-output reviews/task-121/009-phase2-local-50k-100k-slice-prep/artifacts/suite-phase2-local-50k-slice-dryrun-results.jsonl --log-file reviews/task-121/009-phase2-local-50k-100k-slice-prep/artifacts/suite-phase2-local-50k-slice-dryrun.log" reviews/task-121/009-phase2-local-50k-100k-slice-prep/artifacts/suite-phase2-local-50k-slice-dryrun.script.log`
- result: PASS
- key lines:
  - `wrote reviews/task-121/009-phase2-local-50k-100k-slice-prep/artifacts/suite-phase2-local-50k-slice-dryrun-manifest.json`
  - `COMMAND_EXIT_CODE="0"`

### `suite-phase2-local-50k-slice-dryrun-manifest.json`

- dry_run: `true`
- config_sha256: `f4cb6a6c0f019b50835ae6ea480f1001f9b30ac957718e4b226ac0e033eff9f0`
- step count: `50`
- step kinds:
  - `raw`: `1`
  - `load`: `16`
  - `storage`: `16`
  - `recall`: `1`
  - `spire-pipeline`: `16`

Representative generated commands:

- load baseline 50k cell:
  - name: `load-50k_b0_tr10_n128`
  - prefix: `t121_s2_50k_b0_tr10_n128`
  - corpus: `data/staged-current/ec_real_50k_corpus.tsv`
  - storage format: `rabitq`
  - reloptions include `nlists=128`, `boundary_replica_count=0`, `training_sample_rows=10000`
- truth cache 50k:
  - name: `truth-cache-50k-q200-k10`
  - prefix: `t121_s2_50k_b0_tr10_n128`
  - queries limit: `200`
  - truth cache: `reviews/task-121/009-phase2-local-50k-100k-slice-prep/artifacts/50k/truth-cache-50k-q200-k10.json`
- high-coverage/cost 50k interaction cell:
  - name: `pipeline-50k_b4_tr50_n316`
  - prefix: `t121_s2_50k_b4_tr50_n316`
  - sweep: `4,8,12,16,24,32,48,64,96`
  - includes recall, query metrics, cost snapshot, local-store overlap, funnel JSONL, and stage-containment JSONL

## 100k Config

### `suite-phase2-local-100k-slice-dryrun.json`

- derived from: `reviews/task-121/007-phase2-local-factorial-prep/artifacts/suite-phase2-local-factorial-dryrun.json`
- scale: `100k`
- step count: `50`
- cells: `16`
- axes:
  - `boundary_replica_count`: `0`, `1`, `2`, `4`
  - `training_sample_rows`: `10000`, `50000`
  - `nlists`: `128`, `316`
  - `storage_format`: `rabitq`
  - nprobe sweep: `4,8,12,16,24,32,48,64,96`
- non-scope:
  - PQ excluded
  - TurboQuant held for later compatibility/Pareto control

### `suite-phase2-local-100k-slice-dryrun-audit.log`

- command: `script -q -c "target/debug/ecaz bench suite audit --config reviews/task-121/009-phase2-local-50k-100k-slice-prep/artifacts/suite-phase2-local-100k-slice-dryrun.json" reviews/task-121/009-phase2-local-50k-100k-slice-prep/artifacts/suite-phase2-local-100k-slice-dryrun-audit.log`
- result: PASS
- key lines:
  - `[suite:task121-phase2-local-100k-slice-dryrun] audit passed: 50 steps`
  - `COMMAND_EXIT_CODE="0"`

### `suite-phase2-local-100k-slice-dryrun.script.log`

- command: `script -q -c "target/debug/ecaz --database tqvector_bench_task121 --host /home/peter/.pgrx --port 28818 bench suite run --dry-run --config reviews/task-121/009-phase2-local-50k-100k-slice-prep/artifacts/suite-phase2-local-100k-slice-dryrun.json --manifest-output reviews/task-121/009-phase2-local-50k-100k-slice-prep/artifacts/suite-phase2-local-100k-slice-dryrun-manifest.json --results-output reviews/task-121/009-phase2-local-50k-100k-slice-prep/artifacts/suite-phase2-local-100k-slice-dryrun-results.jsonl --log-file reviews/task-121/009-phase2-local-50k-100k-slice-prep/artifacts/suite-phase2-local-100k-slice-dryrun.log" reviews/task-121/009-phase2-local-50k-100k-slice-prep/artifacts/suite-phase2-local-100k-slice-dryrun.script.log`
- result: PASS
- key lines:
  - `wrote reviews/task-121/009-phase2-local-50k-100k-slice-prep/artifacts/suite-phase2-local-100k-slice-dryrun-manifest.json`
  - `COMMAND_EXIT_CODE="0"`

### `suite-phase2-local-100k-slice-dryrun-manifest.json`

- dry_run: `true`
- config_sha256: `2c5062e4fabc7b481b6e1d8c390e0ecee3d49b64cd91d53a7fec5a8e8c56e957`
- step count: `50`
- step kinds:
  - `raw`: `1`
  - `load`: `16`
  - `storage`: `16`
  - `recall`: `1`
  - `spire-pipeline`: `16`

Representative generated commands:

- load baseline 100k cell:
  - name: `load-100k_b0_tr10_n128`
  - prefix: `t121_s2_100k_b0_tr10_n128`
  - corpus: `data/staged-current/ec_real_100k_corpus.tsv`
  - storage format: `rabitq`
  - reloptions include `nlists=128`, `boundary_replica_count=0`, `training_sample_rows=10000`
- truth cache 100k:
  - name: `truth-cache-100k-q200-k10`
  - prefix: `t121_s2_100k_b0_tr10_n128`
  - queries limit: `200`
  - truth cache: `reviews/task-121/009-phase2-local-50k-100k-slice-prep/artifacts/100k/truth-cache-100k-q200-k10.json`
- high-coverage/cost 100k interaction cell:
  - name: `pipeline-100k_b4_tr50_n316`
  - prefix: `t121_s2_100k_b4_tr50_n316`
  - sweep: `4,8,12,16,24,32,48,64,96`
  - includes recall, query metrics, cost snapshot, local-store overlap, funnel JSONL, and stage-containment JSONL

## Staged Inputs

Verified present under `data/staged-current/`:

- `ec_real_50k_corpus.tsv`, `ec_real_50k_queries.tsv`, `ec_real_50k_manifest.json`
- `ec_real_100k_corpus.tsv`, `ec_real_100k_queries.tsv`, `ec_real_100k_manifest.json`

These are local input paths only; no corpus/query TSVs are committed in this packet.
