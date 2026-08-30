# Task 231 Packet 003 artifact manifest

- Head SHA: `1e0d5906abfa5a586091ca51b4ccf1a48690f37f`
- Task/packet: `reviews/task-231/003-format-and-reader/`
- Timestamp: `2026-08-29T19:34:24-07:00`
- Lane: local Intel development host, Rust unit format gate
- Fixture/storage format: pure fixed-stride V1 packed/one-page/multi-block
  byte fixtures; no PostgreSQL relation or benchmark corpus
- Isolation: format-only; no index/table fixture and no shared-table surface

## `fixed-stride-format-tests.log`

- Command: `cargo test -p ecaz fixed_stride`
- SHA-256: `327ebae870d788b2c619e623edb5dfbf27ffdb77a2ae355b0d3602977a40a098`
- Result: `5 passed; 0 failed; 2635 filtered out`
- Covered result lines: packed/one-page/multi-block arithmetic; persisted
  layout re-derivation; generation tag binding; node round-trip and corruption;
  packed and every multi-block page-envelope segment.
