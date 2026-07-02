# Task 121 Phase 2 Local 10k Slice Prep Artifacts

- head_sha: `6f6c803a46ae6e2f2bdd150cceecddb3df42da6a`
- task_bucket: `reviews/task-121`
- packet: `reviews/task-121/008-phase2-local-10k-slice-prep`
- scope: dry-run-only preparation for the Phase 2 local 10k factorial benchmark slice
- timestamp: `2026-06-23T12:49:15Z`
- lane: `intel-local`
- fixture: staged local real corpus at 10k
- storage format: `rabitq`
- rerank mode: default SPIRE pipeline exact-source rerank
- index/table isolation: planned isolated prefix/table/index per 10k factorial cell
- AWS usage: none
- benchmark execution: not run; waiting on Phase 2 sign-off/explicit override

## Config

### `suite-phase2-local-10k-slice-dryrun.json`

- derived from: `reviews/task-121/007-phase2-local-factorial-prep/artifacts/suite-phase2-local-factorial-dryrun.json`
- scale: `10k`
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
  - TurboQuant held for later compatibility/Pareto control, not route-factorial recovery

## Audit

### `suite-phase2-local-10k-slice-dryrun-audit.log`

- command: `script -q -c "target/debug/ecaz bench suite audit --config reviews/task-121/008-phase2-local-10k-slice-prep/artifacts/suite-phase2-local-10k-slice-dryrun.json" reviews/task-121/008-phase2-local-10k-slice-prep/artifacts/suite-phase2-local-10k-slice-dryrun-audit.log`
- result: PASS
- key lines:
  - `[suite:task121-phase2-local-10k-slice-dryrun] audit passed: 50 steps`
  - `COMMAND_EXIT_CODE="0"`

## Dry Run

### `suite-phase2-local-10k-slice-dryrun.script.log`

- command: `script -q -c "target/debug/ecaz --database tqvector_bench_task121 --host /home/peter/.pgrx --port 28818 bench suite run --dry-run --config reviews/task-121/008-phase2-local-10k-slice-prep/artifacts/suite-phase2-local-10k-slice-dryrun.json --manifest-output reviews/task-121/008-phase2-local-10k-slice-prep/artifacts/suite-phase2-local-10k-slice-dryrun-manifest.json --results-output reviews/task-121/008-phase2-local-10k-slice-prep/artifacts/suite-phase2-local-10k-slice-dryrun-results.jsonl --log-file reviews/task-121/008-phase2-local-10k-slice-prep/artifacts/suite-phase2-local-10k-slice-dryrun.log" reviews/task-121/008-phase2-local-10k-slice-prep/artifacts/suite-phase2-local-10k-slice-dryrun.script.log`
- result: PASS
- key lines:
  - `wrote reviews/task-121/008-phase2-local-10k-slice-prep/artifacts/suite-phase2-local-10k-slice-dryrun-manifest.json`
  - `COMMAND_EXIT_CODE="0"`

### `suite-phase2-local-10k-slice-dryrun-manifest.json`

- dry_run: `true`
- config_sha256: `9a64ef3e5de2595da483854863b41ece6941d7d6a83f6421354822e220fa3959`
- step count: `50`
- step kinds:
  - `raw`: `1`
  - `load`: `16`
  - `storage`: `16`
  - `recall`: `1`
  - `spire-pipeline`: `16`

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
  - truth cache: `reviews/task-121/008-phase2-local-10k-slice-prep/artifacts/truth-cache-10k-q200-k10.json`
- high-coverage/cost 10k interaction cell:
  - name: `pipeline-10k_b4_tr50_n316`
  - prefix: `t121_s2_10k_b4_tr50_n316`
  - sweep: `4,8,12,16,24,32,48,64,96`
  - includes recall, query metrics, cost snapshot, local-store overlap, funnel JSONL, and stage-containment JSONL

## Staged Inputs

Verified present under `data/staged-current/`:

- `ec_real_10k_corpus.tsv`
- `ec_real_10k_queries.tsv`
- `ec_real_10k_manifest.json`

These are local input paths only; no corpus/query TSVs are committed in this packet.
