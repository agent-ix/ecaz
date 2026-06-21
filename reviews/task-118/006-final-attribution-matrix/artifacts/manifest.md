# Task 118 Final Attribution Matrix Artifacts

## Packet

- Task bucket: `reviews/task-118/006-final-attribution-matrix`
- Branch: `task-118-hnsw-quantized-recall-attribution`
- Current checkpoint SHA: `1a6e75720b5f10b8999a1d94958e99be39df2eff`
- Timestamp: `2026-06-21T10:51:20-0700`

## Checkpoint: compressed build prefix fix

The initial 10k suite pass exposed an identifier-length failure for the
TurboQuant and PqFastScan compressed-build loads. This checkpoint shortens the
compressed-build `prefix` values in
`crates/ecaz-cli/suites/task118-hnsw-quantized-recall-attribution.json` while
leaving step names and artifact names unchanged.

Artifacts:

- `cargo-test-ecaz-cli-hnsw-prefix-fix.log`
  - Head SHA: `1a6e75720b5f10b8999a1d94958e99be39df2eff`
  - Command: `cargo test -p ecaz-cli hnsw -- --nocapture`
  - Result: `21 passed; 0 failed; 394 filtered out`
- `suite-dry-run-10k-compressed-prefix-fix.log`
  - Head SHA: `1a6e75720b5f10b8999a1d94958e99be39df2eff`
  - Command: `cargo run -p ecaz-cli -- --log-file reviews/task-118/006-final-attribution-matrix/artifacts/suite-dry-run-10k-compressed-prefix-fix.log bench suite run --config crates/ecaz-cli/suites/task118-hnsw-quantized-recall-attribution.json --artifact-dir reviews/task-118/006-final-attribution-matrix/artifacts --only load-10k-hnsw-turboquant-compressed-build --only load-10k-hnsw-pq-fastscan-compressed-build --only load-10k-hnsw-rabitq-compressed-build --dry-run --allow-debug-backend`
  - Result: dry-run expands compressed-build load prefixes to `task118_r10k_tq_cb`, `task118_r10k_pq_cb`, and `task118_r10k_rq_cb`.

## In-progress matrix evidence

The broader 10k/50k/100k attribution matrix remains in progress in this packet.
Do not treat this packet as final closeout until the request cites complete
10k, 50k, and 100k recall, latency, storage, frontier containment, score
correlation, and source-vs-compressed build evidence.
