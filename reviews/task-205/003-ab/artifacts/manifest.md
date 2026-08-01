# Task 205 A/B artifact manifest

- Task bucket/packet: `reviews/task-205/003-ab/`
- Lane/fixture: PG18 `distann-local-multinode`, three physical owner nodes,
  rabitq neighbor codes, default exact co-located rerank.
- Runs: candidate 2026-07-29 22:04 PDT through 23:39 PDT; baseline
  2026-07-29 23:54 PDT through 2026-07-30 00:40 PDT.
- Matrix: 10k/50k/100k, BW=4, H=100, graph degree 32, head cap 4096, head
  width 32, head seed count 32, top-k 10, 200 queries, 50 iterations, 10
  warmups, physical mode, stage counters enabled, fault drills skipped.
- Surface: isolated three-node owner tables per step; no coordinator-resident
  traversal replica; run directories were outside the repository and removed
  after cited results were captured.

## Source and runner provenance

- Candidate implementation checkpoint: `615fd72b2d6d31d7bec9020eabcfa8fa34d39a68`;
  final runner branch head: `c036b70977788e4f04facd3ba02635d84f77fb82`.
- Baseline source: `350736f62` (parent/no-pushdown implementation).
- Both extensions were installed with PG18 feature
  `distann-head-attribution-benchmark` and release profile. The baseline
  release build required a temporary reverted compiler workaround in the
  fault-injection C source; the baseline worktree was clean before execution.
- The suite manifests identify runner head
  `c036b70977788e4f04facd3ba02635d84f77fb82-dirty` because packet/config files
  were present in the runner checkout. The installed extension source is
  identified above by its pinned candidate or baseline checkpoint.

## Inputs

- Staged directory: `/home/peter/dev/ecaz/data/staged-current`.
- `ec_real_10k`: 10,000 corpus rows, 200 queries; corpus SHA-256
  `c67c5810b66d982d705974e48d4775479adfbd92a988f694091266e049a35e75`,
  query SHA-256 `a2c191bb742017d849e73f6e6866e8e0f0bac1579ba212f7fc76b8eb09904ae8`.
- `ec_real_50k`: 50,000 corpus rows, 1,000 queries; corpus SHA-256
  `56023baaa7bc42f758272e8617603d538808e6290a8a70a3a84e057571240133`,
  query SHA-256 `95ac7992578aa80bb193657f10fbcbf1ea3867e559739244bf5a467f7a5a9fa3`.
- `ec_real_100k`: 100,000 corpus rows, 1,000 queries; corpus SHA-256
  `07275cfd5a7a4b415ddf5eacc086de98294ac978532df46ffae30f9202323a95`,
  query SHA-256 `a7cbec6fc44f6c148234538f61339d00d2f10646febc8f667dcbe75d9cf41782`.
- Full hashes and source-row/topology evidence are present in each retained
  `distann-multinode-summary.log` and the suite manifests.

## Configs and commands

- Candidate config: `artifacts/task205-ab-suite.json`; manifest config SHA-256
  `2ade72db2798053950acaf551e90b3f5b032acdda017aaffb5b9f7a287f42f51`.
- Baseline config: `artifacts/task205-baseline-suite.json`; manifest config
  SHA-256 `39450c2c8f1571d4cefb002a9b715d39cc0707fd22686424c325ccea9dc526e8`.
- Candidate command:
  `ecaz bench suite run --config reviews/task-205/003-ab/artifacts/task205-ab-suite.json --artifact-dir reviews/task-205/003-ab/artifacts/run-candidate-stage2 --results-output reviews/task-205/003-ab/artifacts/run-candidate-stage2/results.jsonl --manifest-output reviews/task-205/003-ab/artifacts/run-candidate-stage2/suite-manifest.json --log-file reviews/task-205/003-ab/artifacts/suite-run-candidate-stage2.log`.
- Baseline command:
  `ecaz bench suite run --config reviews/task-205/003-ab/artifacts/task205-baseline-suite.json --artifact-dir reviews/task-205/003-ab/artifacts/run-baseline --results-output reviews/task-205/003-ab/artifacts/run-baseline/results.jsonl --manifest-output reviews/task-205/003-ab/artifacts/run-baseline/suite-manifest.json --log-file reviews/task-205/003-ab/artifacts/suite-run-baseline.log`.
- Shape/input audits passed after staging: `artifacts/audit-candidate.log` (six
  steps) and `artifacts/audit-baseline.log` (three steps).

## Decision-bearing result lines

The retained summaries are the source of truth for all cited values:

- Candidate: `artifacts/run-candidate-stage2/{control-owner-traversal-10k,control-owner-traversal-50k,control-owner-traversal-100k,candidate-algorithm1-10k,candidate-algorithm1-50k,candidate-algorithm1-100k}/distann-multinode-summary.log`.
- Baseline: `artifacts/run-baseline/{baseline-owner-control-10k,baseline-owner-control-50k,baseline-owner-control-100k}/distann-multinode-summary.log`.
- Baseline topology/storage excerpts: `artifacts/baseline-storage-topology.log`.
- Derived NFR calculation: `artifacts/nfr-021-growth.md`.
- Structured suite evidence: the two `results.jsonl` files and corresponding
  `suite-manifest.json` files.

All six suite steps exited 0 and all topology gates passed. The historical raw
fixed-roster growth comparison is retained, but its NFR-021 inadmissibility
verdict is withdrawn: it is not a paper-faithful gate for a fixed roster. The
pushdown counters also show the implementation was inert, so this packet is
superseded by `reviews/task-205/004-l-bounded-rerun/` and is not a
decision-bearing promotion request.
