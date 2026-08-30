# Task 231 Packet 006 artifact manifest

- Head SHA: `fc4a4292681715d899a80d7df251955b5de6f711`.
- Task bucket and packet: `reviews/task-231/006-prepared-lock-lifetime/`.
- Lane: local Intel development host, PostgreSQL 18 / pgrx 0.17.
- Fixture/storage format: isolated fixed-stride EFM1 fixtures with a raw node
  relation and MVCC graph directory; no shared benchmark table.
- Rerank mode: not applicable to this focused mutation-lock correction.

## `fixed-stride-prepared-lock-pg18.log`

- Timestamp: `2026-08-30T02:31:00-07:00`.
- Head SHA: `fc4a4292681715d899a80d7df251955b5de6f711` (the source was committed
  immediately after the green run with no intervening edit).
- Command: `cargo pgrx test pg18 test_distann_fixed_stride`, captured through
  `script -q -e -c` so the command status is preserved.
- SHA-256: `2cbe2557c41b3c289c39a550375f729cee1e26d285a01eb1ce41eff8b6636d50`.
- Result: `4 passed; 0 failed`; exit code 0.
- Key coverage: raw append/overlay/rollback, lifecycle reclaim, seal/topology,
  and two concurrent Repeatable Read writers. The strengthened concurrency
  case holds writer 1's transaction open after its append and requires writer
  2 to finish its append within ten seconds before writer 1 is allowed to
  commit. Both then commit with distinct physical ordinals 1 and 2.

## `clippy-pg18.log`

- Timestamp: `2026-08-30T02:32:00-07:00`.
- Head SHA: `fc4a4292681715d899a80d7df251955b5de6f711`.
- Command: `cargo clippy --lib --no-default-features --features pg18 -- -D
  warnings -A clippy::collapsible-if -A clippy::unnecessary-unwrap`, captured
  through `script -q -e -c`.
- SHA-256: `23a2620c415dc5320232912c5c180c033fc64aadf0e5aad88f6db6a4b64e2559`.
- Result: PASS; exit code 0. The two allowed lints are the same pre-existing
  repository-wide exceptions recorded by Packet 004.

## Decision-run failure evidence

- The durable first-attempt diagnosis is
  `reviews/task-231/005-full-scale-decision/artifacts/run/fixed-stride-10k-a-stall-diagnostic.md`,
  SHA-256:
  `86cdeb31b4cbdb374fd723e06749dea9babdd6ccaabbf9b5fc9404088acba417`.
- It records the exact granted prepared-transaction lock and blocked backlink
  lock on the same raw node-store relation. The incomplete fixture was stopped
  and removed after capture; the final decision matrix has not resumed.
