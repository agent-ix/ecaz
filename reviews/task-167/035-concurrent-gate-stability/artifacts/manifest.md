# Task 167 packet 035 artifacts

- Task bucket: `reviews/task-167/`.
- Packet: `035-concurrent-gate-stability`.
- Harness checkpoint: `d799dc4fe`.
- Trigger: packet 034 synthetic diagnostics crossed the snapshot failure
  boundary but reported controlled target neighbors `6 -> 6`, natural retries
  `0`, and `physical_concurrent_insert_query pass=false`.
- Change isolation: CLI fixture only; no extension product code changed.
- Synthetic contract: deterministic physical-fixture vectors are normalized
  before `encode_to_ecvector`, matching `ecvector_distann_ip_ops` assumptions.
- Capacity contract: a target selected with two open graph slots receives only
  the two controlled writer near-duplicates before saturation is checked.
- Overlap contract: four scanners and two writers share one start barrier;
  scanners run at least 12 queries and continue while either writer is active,
  with a maximum of 192 queries per scanner.
- Validation command:
  `env CARGO_TARGET_DIR=/home/peter/.cargo-target cargo test -p ecaz-cli task167_ --no-default-features --quiet`.
- Validation result: 2 passed, 0 failed, 497 filtered in
  `validation-test.log` (SHA-256
  `79fa5b0a855c1269f0f4ccf3af0ba61ba0f15f200c9b77f07d99b533b479e2fa`).
- Runtime status: pending clean-head release rebuild/install and synthetic-only
  suite rerun. No accepted concurrency, retry, saturation, recall, latency, or
  storage result is claimed yet.
