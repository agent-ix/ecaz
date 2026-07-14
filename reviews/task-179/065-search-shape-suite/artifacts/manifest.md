# Artifact manifest

- Head SHA: `a7fa64895`
- Task bucket / packet: `reviews/task-179/065-search-shape-suite`
- Lane: local CLI validation
- Fixture / storage / rerank: suite expansion only; no measurement
- Isolation: focused CLI tests

## `check.log`

- SHA-256: `376889c59832ac9ff00f1f974a8818c3005af9d27f0f4b99c93831b466d043e8`
- Command: `cargo check -p ecaz-cli`
- Result: PASS (one pre-existing `LoadedDistributedPlacementConfig.path` warning)

## `tests.log`

- SHA-256: `49585726dfaea19fb1eaecbefb4eba63fe8f32a05908d52d97d00ac5179e63d0`
- Command: `cargo test -p ecaz-cli distann_local_multinode_ -- --nocapture`
- Result: PASS (`3 passed; 0 failed; 432 filtered out`)
