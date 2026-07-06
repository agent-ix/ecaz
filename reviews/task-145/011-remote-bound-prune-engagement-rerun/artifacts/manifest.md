# Task 145 Packet 011 Artifact Manifest

- head SHA: `ef0ffd5adff02b605efb01855876af27ff5b5d14`
- task bucket: `reviews/task-145/011-remote-bound-prune-engagement-rerun/`
- generated: `2026-07-06T16:08:10-07:00`
- lane: local multinode PG18, release install, remote production-read path
- runner: `target/release/ecaz bench suite`
- suite config: `artifacts/task145-remote-bound-prune-engagement-rerun-suite.json`
- top-level suite manifest: `artifacts/suite-manifest-r2.json`
- top-level suite results: `artifacts/suite-results-r2.jsonl` (empty because nested `spire-local-multinode` suites emit the measurement rows)
- validity gate: `bound-prune-off` must have `pre_materialization_pruned_sum=0` and `bound-prune-on` must have `pre_materialization_pruned_sum>0` before latency/recall interpretation is valid.

## Feedback Processed

- `reviews/task-145/008-remote-bound-prune-ab/feedback/2026-07-06-01-agent-ix.md`: packet 008 was rejected as an inert/null A/B, not a measured negative.
- `reviews/task-145/010-bound-prune-engagement-counter/feedback/2026-07-06-01-agent-ix.md`: packet 010 instrumentation was approved, but the counter-instrumented A/B was still owed.

## Invalid Excluded Run

- `remote-10k-n128-bound-r1` was aborted before completion and is excluded. It used a stale `target/release/ecaz` CLI that did not render `pre_materialization_pruned_sum`, so it cannot answer the engagement question.
- No r1 artifact is cited as evidence in this packet.

## Commands

Audit:

```text
target/release/ecaz bench suite audit \
  --config reviews/task-145/011-remote-bound-prune-engagement-rerun/artifacts/task145-remote-bound-prune-engagement-rerun-suite.json \
  --log-file reviews/task-145/011-remote-bound-prune-engagement-rerun/artifacts/suite-audit-r2.log
```

Dry run:

```text
target/release/ecaz bench suite run \
  --dry-run \
  --config reviews/task-145/011-remote-bound-prune-engagement-rerun/artifacts/task145-remote-bound-prune-engagement-rerun-suite.json \
  --artifact-dir reviews/task-145/011-remote-bound-prune-engagement-rerun/artifacts \
  --manifest-output reviews/task-145/011-remote-bound-prune-engagement-rerun/artifacts/suite-manifest-dry-run-r2.json \
  --results-output reviews/task-145/011-remote-bound-prune-engagement-rerun/artifacts/suite-results-dry-run-r2.jsonl \
  --log-file reviews/task-145/011-remote-bound-prune-engagement-rerun/artifacts/suite-dry-run-r2.log
```

Run:

```text
target/release/ecaz bench suite run \
  --config reviews/task-145/011-remote-bound-prune-engagement-rerun/artifacts/task145-remote-bound-prune-engagement-rerun-suite.json \
  --artifact-dir reviews/task-145/011-remote-bound-prune-engagement-rerun/artifacts \
  --manifest-output reviews/task-145/011-remote-bound-prune-engagement-rerun/artifacts/suite-manifest-r2.json \
  --results-output reviews/task-145/011-remote-bound-prune-engagement-rerun/artifacts/suite-results-r2.jsonl \
  --log-file reviews/task-145/011-remote-bound-prune-engagement-rerun/artifacts/suite-run-r2.log
```

## Matrix

All cells used `storage_format=rabitq`, `boundary_replica_count=0`, `source_identity=include`, `top_k=10`, `queries_limit=200`, `sweep=8,16,32,64,96`, and production-read variants:

- `bound-prune-off`: `ec_spire.pre_materialization_prune=off`
- `bound-prune-on`: `ec_spire.pre_materialization_prune=on`

Cells:

- `remote-10k-n128-bound-r2`: `ec_real_10k`, `nlists=128`, run id `t145bpeng2-10n128`
- `remote-50k-n1024-bound-r2`: `ec_real_50k`, `nlists=1024`, run id `t145bpeng2-50n1024`
- `remote-100k-n1024-bound-r2`: `ec_real_100k`, `nlists=1024`, run id `t145bpeng2-100n1024`

## Release Proof

Source: `artifacts/release-profile-summary.txt`.

Each cell records:

- `install_profile=release`
- node build profile `release` for coord, remote1, remote2, remote3
- `HARNESS PASSED`

## Engagement Result

Source: `artifacts/engagement-summary.txt`, extracted only from the `Production selected-leaf scan profile` table in each production-read log.

```text
10k-n128-bound-r2 bound-prune-off rows=15 leaf_candidate_sum=3410123 truncated_sum=3380433 pre_materialization_pruned_sum=0 pre_materialization_pruned_max=0 sound_bound_available_sum=0 sound_bound_missing_sum=43200
10k-n128-bound-r2 bound-prune-on rows=15 leaf_candidate_sum=3410123 truncated_sum=3380433 pre_materialization_pruned_sum=0 pre_materialization_pruned_max=0 sound_bound_available_sum=0 sound_bound_missing_sum=43200
50k-n1024-bound-r2 bound-prune-off rows=15 leaf_candidate_sum=2221377 truncated_sum=2191497 pre_materialization_pruned_sum=0 pre_materialization_pruned_max=0 sound_bound_available_sum=0 sound_bound_missing_sum=43200
50k-n1024-bound-r2 bound-prune-on rows=15 leaf_candidate_sum=2221377 truncated_sum=2191497 pre_materialization_pruned_sum=0 pre_materialization_pruned_max=0 sound_bound_available_sum=0 sound_bound_missing_sum=43200
100k-n1024-bound-r2 bound-prune-off rows=15 leaf_candidate_sum=4167023 truncated_sum=4137373 pre_materialization_pruned_sum=0 pre_materialization_pruned_max=0 sound_bound_available_sum=0 sound_bound_missing_sum=43200
100k-n1024-bound-r2 bound-prune-on rows=15 leaf_candidate_sum=4167023 truncated_sum=4137373 pre_materialization_pruned_sum=0 pre_materialization_pruned_max=0 sound_bound_available_sum=0 sound_bound_missing_sum=43200
```

Interpretation: the mechanism did not engage in any `bound-prune-on` cell. The on and off arms are identical on candidate/truncation counters, and the dedicated prune counter is zero. Therefore the suite is null evidence for latency/recall economy and only proves runtime inertness under this configuration.

## Cited Artifacts

- `artifacts/engagement-summary.txt`
- `artifacts/release-profile-summary.txt`
- `artifacts/suite-audit-r2.log`
- `artifacts/suite-dry-run-r2.log`
- `artifacts/suite-manifest-dry-run-r2.json`
- `artifacts/suite-run-r2.log`
- `artifacts/suite-manifest-r2.json`
- `artifacts/task145-remote-bound-prune-engagement-rerun-suite.json`
- `artifacts/remote-10k-n128-bound-r2/local-multinode.log`
- `artifacts/remote-10k-n128-bound-r2/bench-suite/suite-manifest.json`
- `artifacts/remote-10k-n128-bound-r2/bench-suite/results.jsonl`
- `artifacts/remote-10k-n128-bound-r2/bench-suite/production-read-k10-bound-prune-off-default.log`
- `artifacts/remote-10k-n128-bound-r2/bench-suite/production-read-k10-bound-prune-on-default.log`
- `artifacts/remote-50k-n1024-bound-r2/local-multinode.log`
- `artifacts/remote-50k-n1024-bound-r2/bench-suite/suite-manifest.json`
- `artifacts/remote-50k-n1024-bound-r2/bench-suite/results.jsonl`
- `artifacts/remote-50k-n1024-bound-r2/bench-suite/production-read-k10-bound-prune-off-default.log`
- `artifacts/remote-50k-n1024-bound-r2/bench-suite/production-read-k10-bound-prune-on-default.log`
- `artifacts/remote-100k-n1024-bound-r2/local-multinode.log`
- `artifacts/remote-100k-n1024-bound-r2/bench-suite/suite-manifest.json`
- `artifacts/remote-100k-n1024-bound-r2/bench-suite/results.jsonl`
- `artifacts/remote-100k-n1024-bound-r2/bench-suite/production-read-k10-bound-prune-off-default.log`
- `artifacts/remote-100k-n1024-bound-r2/bench-suite/production-read-k10-bound-prune-on-default.log`

Excluded from commit: generated corpus TSVs, correctness TSVs, server logs, load logs, materialization logs, remote identity files, registration logs, and aborted r1 artifacts.
