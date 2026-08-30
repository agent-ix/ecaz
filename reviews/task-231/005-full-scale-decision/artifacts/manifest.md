# Task 231 Packet 005 preregistration artifact manifest

- Head SHA: `bf4b78ed2ad462e6c15816fa6544dfd46ee7414c`.
- Task bucket and packet: `reviews/task-231/005-full-scale-decision/`.
- Lane: local Intel development host, PostgreSQL 18 target.
- Fixture/storage formats: isolated fresh current-heap control versus
  fixed-stride candidate; one index/table surface per suite step.
- Rerank/search mode: production RaBitQ neighbor scoring with exact-vector
  materialization, BW4/H100 primary and BW16/H25 transfer pair.
- Measurement state: preregistration only; no A/B result exists yet.
- Suite config:
  `crates/ecaz-cli/suites/task231-fixed-stride-10k-50k-100k.json`, SHA-256
  `48dbcbf38383d99418e99b6f246149c5fb7b552b696444ed6cd8e9379da1d211`.

## `preregistration-audit.log`

- Timestamp: `2026-08-30T00:26:58-07:00`.
- Head SHA: `bf4b78ed2ad462e6c15816fa6544dfd46ee7414c`.
- Command: `/home/peter/.cargo-target/debug/ecaz bench suite audit --config
  crates/ecaz-cli/suites/task231-fixed-stride-10k-50k-100k.json --log-file
  reviews/task-231/005-full-scale-decision/artifacts/preregistration-audit.log`.
- SHA-256: `1297d037ee0c2d1c86607585e52c35cabad8ee9c8f63163abac022136568c989`.
- Key result: `audit passed: 27 steps`; exit code 0.

## `preregistration-cli-tests.log`

- Timestamp: `2026-08-30T00:25:16-07:00`.
- Head SHA: `9bb9e0f1b15996389b41e3b872bf1ba68ebd97f2`.
- Command: `cargo test -p ecaz-cli task231` (captured through
  `script -q -e -c`).
- SHA-256: `eb0373b61cf1a86b5fb8b04fd34c5666c58fbcedb57ac1e74efbd29b50e2f1d3`.
- Key result: `2 passed; 0 failed`; exit code 0. The tests cover fixed-stride
  suite expansion/cold-profile validation and structured parsing of the
  checksum plus DML raw-store-growth metrics.
