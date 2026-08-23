# Task 167 packet 049 artifact manifest

- Head under review:
  `9d0f095171fbd4a2eb7272f660eafcbbe4d09337` — preserve Task 167 failed
  quality rows and summaries before returning the hard-gate error.
- Formatter-only companion:
  `30dc2e5b73d26ba5657cacffed8ab176a9715afb` — rustfmt line wrapping in
  `commands/bench/latency.rs`; the operator explicitly approved retaining this
  formatting change.
- Owning packet: `reviews/task-167/049-preserve-quality-failure-summary/`.
- Timestamp: `2026-08-22`.
- Scope: harness control flow only; no PostgreSQL fixture or benchmark rerun.

## Validation

- Command:
  `cargo test -p ecaz-cli --no-default-features task167_quality_gate`.
- Result: passed, 2/2; 509 filtered out. Artifact: `cargo-test.log`, committed
  LF-normalized SHA-256
  `e44591797e98a97c0a868a16a0768873ecca41867809586486e59892a4e34ec3`.
- Command: `cargo check -p ecaz-cli --no-default-features`.
- Result: passed. The only warning is the pre-existing unused `path` field at
  `commands/corpus/load.rs:190`. Artifact: `cargo-check.log`, committed
  LF-normalized SHA-256
  `1b1a152f3e3b43effa62256ee8fdca206f381260f1fccc0fb643eaf91a86bb44`.
- Static check: `git diff --check` passed before the code checkpoint.
