# Task 119 Sidecar Rerank Matrix Support Artifacts

- Task bucket: `reviews/task-119/002-sidecar-rerank-matrix-support`
- Checkpoint SHA: `5f9120c44cd6d696745e5c165135cd8cfb2ad2a4`
- Host/lane: M5 laptop local development checkout
- Scope: CLI and suite-runner support for the required Task 119 second-stage
  rerank matrix over existing HNSW RaBitQ candidate-frontier prefixes.
- Benchmark status: support/dry-run only. This packet does not contain the
  required 10k/50k/100k measured result rows.

## Code and Config

- `crates/ecaz-cli/src/commands/bench/sidecar_rerank.rs`
  - adds `rabitq2` and `rabitq4` sidecar variants alongside existing
    `rabitq8`;
  - adds `turboquant_2bit`, `turboquant_3bit`, `turboquant_4bit`,
    `turboquant_5bit`, `turboquant_6bit`, `turboquant_7bit`, and
    `turboquant_8bit`;
  - scores TurboQuant sidecar payloads through `ProdQuantizer::score_ip_encoded`;
  - keeps existing `f32`, `f16`, `rabitq8ls`, `rabitq8c3`, and `rabitq8c4`
    variants available for non-Task-119 studies.
- `crates/ecaz-cli/suites/task119-hnsw-rabitq-sidecar-rerank-matrix.json`
  - enumerates one `sidecar-rerank` step per scale: 10k, 50k, 100k;
  - uses `profile = ec_hnsw`, `candidate_k = 1000`, `sweep = [320, 500, 1000]`;
  - targets `task119_real{10,50,100}k_hnsw_rabitq` prefixes;
  - explicitly lists the required variants:
    `f32`, `rabitq2`, `rabitq4`, `rabitq8`,
    `turboquant_2bit`, `turboquant_3bit`, `turboquant_4bit`,
    `turboquant_5bit`, `turboquant_6bit`, `turboquant_7bit`,
    `turboquant_8bit`.

## Validation

### `cargo check -p ecaz-cli`

- Artifact: `cargo-check-ecaz-cli.log`
- Result: succeeded.
- Key line: `Finished dev profile [unoptimized + debuginfo] target(s)`

### `cargo test -p ecaz-cli sidecar -- --nocapture`

- Artifact: `cargo-test-ecaz-cli-sidecar.log`
- Result: succeeded.
- Key line: `test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 411 filtered out`

### Suite Audit

Command:

```text
./target/debug/ecaz bench suite audit --config crates/ecaz-cli/suites/task119-hnsw-rabitq-sidecar-rerank-matrix.json --log-file reviews/task-119/002-sidecar-rerank-matrix-support/artifacts/suite-audit.log
```

- Artifact: `suite-audit.log`
- Result: succeeded.
- Key line: `[suite:task119-hnsw-rabitq-sidecar-rerank-matrix] audit passed: 3 steps`

### Suite Dry Run

Command:

```text
./target/debug/ecaz bench suite run --dry-run --config crates/ecaz-cli/suites/task119-hnsw-rabitq-sidecar-rerank-matrix.json --manifest-output reviews/task-119/002-sidecar-rerank-matrix-support/artifacts/suite-manifest.dry-run.json --log-file reviews/task-119/002-sidecar-rerank-matrix-support/artifacts/suite-dry-run.log
```

- Artifacts: `suite-dry-run.log`, `suite-manifest.dry-run.json`
- Result: succeeded.
- Key line: dry-run manifest contains 3 steps, each with all required Task 119
  variants listed explicitly.

## Known Remaining Work

- Run the new suite against the release benchmark database or a fresh isolated
  M5 benchmark database.
- Store measured `suite-results.jsonl` plus sidecar output logs under a follow-up
  Task 119 packet.
- Promote or shelve only after the measured rows cover 10k/50k/100k recall,
  latency, sidecar bytes/storage, and candidate frontier behavior for every
  required rerank representation.
