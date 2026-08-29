# Task 230 packet 005 artifact manifest

- Head SHA: `0b15cf02096daca8a32fb94ed11b5b778360ee45`
- Task bucket: `reviews/task-230/005-suite-runner-attribution-count/`
- Timestamp: 2026-08-29T07:03:10-07:00
- Lane / fixture / storage format / rerank mode: local Intel PG18;
  Packet 004 first 10k row-heap control; descriptor V4 / Graph V2; no rerank
- Isolation: one failed fresh fixture under
  `/home/peter/.ecaz/clusters/task230-10k-pair-a-rowheap-first`; no result is
  admitted and the fixture will be removed before the clean restart

## `failed-first-arm-summary.log`

- Source command: `/home/peter/.cargo-target/debug/ecaz bench suite run
  --config crates/ecaz-cli/suites/task230-hot-cold-10k-50k-100k.json`
- Source log: packet 004 `artifacts/suite-console.log` at SHA-256
  `1140458edd463eb7b5d49667260c19959886d2ea3567a842f5d92665513db048`.
- Key result: release preflight passed at `35648e467`, then the runner rejected
  62 emitted attribution-work rows against its stale expected count of 52.

## `cargo-test-runner-counter-fix.log`

- Command: `cargo test -p ecaz-cli
  task230_io_projections_mirror_all_six_end_to_end_shapes`
- Result: exit 0; 1 passed, 0 failed, 551 filtered out.

## Source inspection

- `src/am/ec_distann/stage_counters.rs` declares
  `DistannMaterializationWork::ALL: [Self; 61]`.
- `crates/ecaz-cli/src/commands/bench/latency.rs` appends one
  `client_result_rows` snapshot whenever distann stage counters are enabled.
- `crates/ecaz-cli/src/commands/dev/distann_multicluster.rs` now enforces 62
  rows per concurrency group.
