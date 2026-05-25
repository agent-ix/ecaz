# Task 59 Packet 007 Artifact Manifest

- head SHA: `d3a99b2ab6ee43075a1a49ee25478cc86e90aee0`
- task bucket: `reviews/task-59/007-cache-state-latency`
- packet path: `reviews/task-59/007-cache-state-latency`
- timestamp: `2026-05-25T02:06:18Z`
- lane: AWS Graviton DiskANN benchmark runner instrumentation
- fixture: `task59-aws-diskann-final-graviton-suite`
- storage format: `pq_fastscan`
- rerank mode: default DiskANN scan path, no sidecar rerank
- isolated one-index-per-table: yes

## Artifacts

### `cargo-test-ecaz-cli-latency.log`

- command: `script -q -e -c 'cargo test -p ecaz-cli commands::bench::latency' reviews/task-59/007-cache-state-latency/artifacts/cargo-test-ecaz-cli-latency.log`
- result: pass
- key line: `test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 347 filtered out`

### `cargo-test-ecaz-cli-suite-cache-state.log`

- command: `script -q -e -c 'cargo test -p ecaz-cli commands::bench::suite::tests::expands_latency_with_cache_state_label' reviews/task-59/007-cache-state-latency/artifacts/cargo-test-ecaz-cli-suite-cache-state.log`
- result: pass
- key line: `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 361 filtered out`

### `jq-suite-json.log`

- command: `script -q -e -c 'jq empty benchmarks/task59-aws-diskann-final-graviton-suite/suite.json' reviews/task-59/007-cache-state-latency/artifacts/jq-suite-json.log`
- result: pass
- key line: command exited successfully with no output

### `jq-suite-1m-resume-json.log`

- command: `script -q -e -c 'jq empty benchmarks/task59-aws-diskann-final-graviton-suite/suite-1m-resume.json' reviews/task-59/007-cache-state-latency/artifacts/jq-suite-1m-resume-json.log`
- result: pass
- key line: command exited successfully with no output
