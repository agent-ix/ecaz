# Task 120 Packet 011 Artifact Manifest

- benchmark head SHA: `7f15698391984085169311646bc63cc4983d52fd`
- task bucket: `reviews/task-120/`
- packet path: `reviews/task-120/011-phase4-route-overfetch/`
- lane: local PG18 Intel host
- database/socket: `tqvector_bench_task120` on `/home/peter/.pgrx`, port `28818`
- fixture: staged real corpus at `data/staged-current/ec_real_{10k,50k,100k}_corpus.tsv`
- access method / quantizer: `ec_spire` recursive RabitQ f8/b64/L2
- surfaces: isolated one-index-per-table prefixes reused from packet 008
  (`task120_phase2_real{10k,50k,100k}_spire_rabitq_f8_b64_l2`), not shared-table
- rerank mode: default `rerank_width=25`
- route/topology modes: route overfetch sweep `nprobe=32,48,64,96`; route-row
  budget variants at `nprobe=96` with `max_routed_candidate_rows=25000,50000,75000`
- route ceiling: staged indexes have `top_graph_search_list_size=96`, so
  `nprobe=128` was excluded rather than measured against incompatible reloptions
- remote/distributed: false; this is local topology route-set evidence only
- run timestamp: `2026-06-21 13:13:19 -0700` to `2026-06-21 13:27:56 -0700`
  (`suite-results.jsonl` mtime); report generated `2026-06-21 13:28:08 -0700`
- corpus data: TSV corpus/query/truth inputs were not committed; the suite reads
  staged local data and records the file paths in command lines

## Host And Runner

- `precheck-host.log`
  - command: recorded by `suite-status.log` as `dev sql --pg 18 ...`
  - key result: PostgreSQL `18.3`, `ecaz_build_profile=release`,
    `ec_spire.nprobe=-1`, `ec_spire.rerank_width=-1`,
    `ec_spire.max_candidate_rows=-1`, `ec_spire.max_routed_candidate_rows=0`,
    `ec_spire.adaptive_nprobe=off`
- `suite.json`
  - checked-in task-local `SuiteConfig`
  - config SHA256: `75449f98f6b0b7a681a6cc1b333ac067d283c63537e2f9a699f19efea9899e42`
  - steps: 1 host precheck, 3 storage probes, 12 pipeline route variants
  - query limit: 200 queries per scale
  - recall source: `--truth-corpus-file data/staged-current/ec_real_*_corpus.tsv`
- `suite-audit.log`
  - command: `target/debug/ecaz bench suite audit --config reviews/task-120/011-phase4-route-overfetch/artifacts/suite.json --database tqvector_bench_task120 --host /home/peter/.pgrx --port 28818 --log-file reviews/task-120/011-phase4-route-overfetch/artifacts/suite-audit.log`
  - key result: `audit passed: 16 steps`
- `suite-dry-run.log`, `suite-manifest.dry-run.json`
  - command: `target/debug/ecaz bench suite run --dry-run --config reviews/task-120/011-phase4-route-overfetch/artifacts/suite.json --database tqvector_bench_task120 --host /home/peter/.pgrx --port 28818 --manifest-output reviews/task-120/011-phase4-route-overfetch/artifacts/suite-manifest.dry-run.json --log-file reviews/task-120/011-phase4-route-overfetch/artifacts/suite-dry-run.log`
- `suite-run.log`, `suite-manifest.json`, `suite-results.jsonl`
  - command: `target/debug/ecaz bench suite run --config reviews/task-120/011-phase4-route-overfetch/artifacts/suite.json --database tqvector_bench_task120 --host /home/peter/.pgrx --port 28818 --manifest-output reviews/task-120/011-phase4-route-overfetch/artifacts/suite-manifest.json --results-output reviews/task-120/011-phase4-route-overfetch/artifacts/suite-results.jsonl --log-file reviews/task-120/011-phase4-route-overfetch/artifacts/suite-run.log`
- `suite-status.log`
  - command: `target/debug/ecaz bench suite status --manifest reviews/task-120/011-phase4-route-overfetch/artifacts/suite-manifest.json --log-file reviews/task-120/011-phase4-route-overfetch/artifacts/suite-status.log`
  - key result: `completed=16 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0`
- `suite-report.md`, `suite-report-results.jsonl`
  - command: `target/debug/ecaz bench suite report --manifest reviews/task-120/011-phase4-route-overfetch/artifacts/suite-manifest.json --results-output reviews/task-120/011-phase4-route-overfetch/artifacts/suite-report-results.jsonl --log-file reviews/task-120/011-phase4-route-overfetch/artifacts/suite-report.md`
- `phase4-route-overfetch-summary.txt`
  - compact packet-local source for the route-overfetch and row-budget decision tables

## Per-Scale Artifacts

For each scale, the packet includes:

- `storage-{10k,50k,100k}-recursive-rabitq.log`
- `pipeline-{10k,50k,100k}-overfetch.log`
- `pipeline-{10k,50k,100k}-overfetch96-rowcap25k.log`
- `pipeline-{10k,50k,100k}-overfetch96-rowcap50k.log`
- `pipeline-{10k,50k,100k}-overfetch96-rowcap75k.log`

No raw per-query pipeline JSONL families are committed in this packet. The
committed JSONL files are the suite runner's structured `suite-results.jsonl`
and `suite-report-results.jsonl`.

## Storage And Reloptions

| Scale | Rows | Total | Total bytes | ec_spire index | Index bytes | Top graph search list |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 10k | 10,000 | 168.8 MiB | 176,999,629 | 9.7 MiB | 10,171,187 | 96 |
| 50k | 50,000 | 837.0 MiB | 877,658,112 | 42.1 MiB | 44,145,050 | 96 |
| 100k | 100,000 | 1.6 GiB | 1,717,986,918 | 82.5 MiB | 86,507,520 | 96 |

## Key Result Lines

| Scale | Variant | recall@10 | p50 | p95 | routes | candidate_sum | object_bytes_sum |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 50k | nprobe32 | 0.9725 | 15.032 ms | 17.371 ms | 6,400 | 2,326,779 | 1,960,850,458 |
| 50k | nprobe96 | 1.0000 | 33.494 ms | 35.887 ms | 19,200 | 7,008,867 | 5,906,826,386 |
| 50k | nprobe96-rowcap25k | 1.0000 | 32.172 ms | 34.423 ms | 13,799 | 5,043,969 | 4,250,896,754 |
| 100k | nprobe32 | 0.9310 | 26.121 ms | 32.602 ms | 6,400 | 5,165,224 | 4,344,876,152 |
| 100k | nprobe96 | 0.9975 | 66.596 ms | 96.757 ms | 19,200 | 15,506,227 | 13,043,852,590 |
| 100k | nprobe96-rowcap25k | 0.9975 | 63.595 ms | 93.190 ms | 6,315 | 5,109,734 | 4,298,195,094 |

## Decision

- Plain route overfetch is recall-positive but expensive.
- A 25k routed-row budget preserves the nprobe 96 recall at 50k and 100k while
  cutting routed/object volume substantially.
- Looser 50k/75k routed-row caps do not improve recall and add work at 100k.
- Go/no-go: carry route overfetch plus a tight routed-row budget forward as a
  candidate Phase 5/AWS hypothesis. Do not promote a product default from this
  local-only packet.

## Guardrails

- This packet is not Task 120 closeout. Phases 5-6 and AWS/distributed evidence
  remain open.
- This packet does not claim a code change, storage improvement, or merge-ready
  product default.
- The suite used `ecaz bench suite`; no ad-hoc sweeper was added.
