# Task 230 packet 007 artifact manifest

- Head SHA: `93615542d976410a68ab2a438f428f437e21f8c9`
- Task bucket: `reviews/task-230/007-hot-cold-owner-probe/`
- Timestamp: 2026-08-29T08:12:27-07:00
- Lane / fixture / storage format / rerank mode: local Intel PG18; first 10k
  hot/cold primary arm; descriptor V4 / Graph V2; no rerank
- Isolation: one fresh hot/cold fixture; failure occurred after topology and
  serving but before recall/latency/storage/DML/I/O measurement

## `failed-hotcold-owner-probe-summary.log`

- Source command: frozen Packet 004 suite at config SHA-256
  `e141ac65a7e18eaf4512509c549ba750e3106a2a045942e0eb6a5ac8fcc5437c`.
- Key result: release preflight, hot/cold Ready/Published topology, and serving
  passed at `7718eb4b`; the remote-owner sample then failed because the compact
  hot relation has no logical `source_id` column.
- Result disposition: no hot/cold benchmark row admitted; the earlier row-heap
  result is also discarded because the corrected rerun must use one CLI head.

## Validation artifacts

- `cargo-fmt-check.log`: `cargo fmt --check`, exit 0.
- `cargo-test-focused.log`: focused physical-identity mapping test, exit 0;
  1 passed, 0 failed, 552 filtered out.
- `cargo-clippy-cli.log`: CLI all-targets clippy, exit 0; 77 binary / 78 test
  warnings, equal to the frozen baseline.
