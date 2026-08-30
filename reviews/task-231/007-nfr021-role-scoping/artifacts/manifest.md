# Task 231 Packet 007 artifact manifest

- Head SHA: `795af9616a304f2bf276d57c2c151270198f9bd4`.
- Task bucket and packet:
  `reviews/task-231/007-nfr021-role-scoping/`.
- Scope: benchmark-runner derived NFR-021 evidence aggregation only; no
  extension or fixture behavior changed.
- Measurement source: Packet 005's all-succeeded 27-step manifest at accepted
  extension SHA `66b53998a955b583ca43c0e967806aa29e0a4404`.

## `focused-test.log`

- Timestamp: `2026-08-30T12:07:39-07:00`.
- Command: `cargo test -p ecaz-cli
  distann_nfr_021_same_variant_does_not_mix_decision_roles` (captured through
  `script -q -e -c`).
- SHA-256:
  `232ee18345c292d2bfb2a6d4f8d6fcf35377cb183acb6e14d058bbf9a271c1ba`.
- Key result: `1 passed; 0 failed`; exit code 0.

