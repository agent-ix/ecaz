# Task 167 packet 046 artifact manifest

- Head SHA: `c3b01290b939b54315b6e1970b80cc68774d55d0`.
- Task bucket and packet:
  `reviews/task-167/046-shipped-quality-gate-isolation/`.
- Lane: ecaz-cli Task 167 distributed DistANN benchmark harness, PG18 primary
  runtime target, shipped robust-prune insert mode.
- Fixture / storage / rerank: no runtime fixture in this code checkpoint; no
  storage or rerank measurement is claimed. Packet 045 owns the calibration
  evidence used for the hard bands.
- Isolation surface: the code now measures one physical shipped-default insert
  arm before the diagnostic candidate; runtime confirmation will use one index
  per table in the next suite packet.
- Timestamp: `2026-08-22`.

## Artifacts

### `task167-cli-tests.log`

- Command: `cargo test -p ecaz-cli task167_ --no-default-features`.
- Result: passed; 10 passed, 0 failed, 497 filtered out.
- SHA-256:
  `e7d48febb9874fb602ebad210dfc53f6bd0f7b178349812495f221ef6317143a`.

### `ecaz-cli-check.log`

- Command: `cargo check -p ecaz-cli`.
- Result: passed; one pre-existing unrelated dead-code warning at
  `crates/ecaz-cli/src/commands/corpus/load.rs:190`.
- SHA-256:
  `2432c188210d1e2875a4a4f06391fe3659b4fde1df11a0ea76747cc6e8e2417a`.

No corpus data, PostgreSQL operational logs, cluster state, truth cache, or
polling output is included.
