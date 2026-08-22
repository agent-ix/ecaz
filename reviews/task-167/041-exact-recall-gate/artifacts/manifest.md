# Task 167 packet 041 artifacts

- Task bucket: `reviews/task-167/`.
- Packet: `041-exact-recall-gate`.
- Code checkpoint: `f83110078ec287060c1d2f714a17835084b3bd6c`.
- Feedback addressed:
  `reviews/task-167/039-post-insert-parity-gate/feedback/2026-08-22-01-reviewer.md`
  sections 1–4, 6, 7.2, and 8. Packet 040 addresses sections 5 and 7.3.
- Change isolation: benchmark/fixture harness only; no index algorithm or
  storage-format change.
- Validation command:
  `env CARGO_TARGET_DIR=/home/peter/.cargo-target cargo test -p ecaz-cli task167_ --no-default-features --quiet`.
- Validation result: 9 passed, 0 failed, 497 filtered in
  `cli-validation-test.log` (SHA-256
  `f40e5ad05bb99dbf0c43e6aeea61a6e9d2c1c769ce6dc9d914a1fbbe24543e18`).
- Static whitespace validation: `git diff --check` passed before commit.
- Release CLI build command:
  `env CARGO_TARGET_DIR=/home/peter/.cargo-target cargo build -p ecaz-cli --release --no-default-features`.
- Release CLI build result: passed at the exact code checkpoint in
  `build-cli.log` (SHA-256
  `1042e08cce2ef2ab949eca02108e7e7606f9c72f22f2823a09c9f5188919830c`);
  one pre-existing dead-code warning names
  `commands/corpus/load.rs:190` and is unrelated to this change.
- Repository-wide `cargo fmt --all -- --check` is not used as packet evidence:
  the stable toolchain reports pre-existing formatting differences across
  untouched files because the repository config requests nightly-only import
  grouping. Packet changes remain formatter-generated and `git diff --check`
  clean.
- Suite config: `task167-exact-recall-suite.json` (SHA-256
  `05306ab63dd6e142189259d5a4ab456a0e4bf10181379d292d61a7e8af8cb267`).
- Suite audit command:
  `/home/peter/.cargo-target/release/ecaz bench suite audit --config reviews/task-167/041-exact-recall-gate/artifacts/task167-exact-recall-suite.json --log-file reviews/task-167/041-exact-recall-gate/artifacts/suite-audit.log`.
- Suite audit result: passed, 3 steps, in `suite-audit.log` (SHA-256
  `8dbf37fcaa5e63d9a418affa5a264eddc33d12cf25c56fa44e989e58d1087936`).
- Suite query populations per scale: 48 inserted-neighborhood plus 152
  held-out; held-out dominates.
- Exact truth: brute-force fp32 inner product over the staged corpus plus the
  exact 320 physical insert rows. Corpus IDs must be strictly increasing so
  file row order matches SQL `ORDER BY id OFFSET` selection.
- Duplicate handling: SHA-256 of source f32 bit patterns; per-query denominator
  is the number of distinct exact-truth source fingerprints. Aggregate truth
  slots, distinct keys, and duplicate slots are emitted separately.
- Compared access paths: incremental physical distributed scan versus
  reloption-matched local fresh rebuild; the first query in each arm must
  contain the expected plan node.
- Quality gate: physical distinct recall may trail fresh distinct recall by at
  most 0.002 independently for both populations.
- Insert sample evidence: every successful statement is counted at runtime;
  the preregistered and observed count must both equal 160 for the control,
  append-disabled, and append-enabled arms.
- Synthetic normalization preflight: PostgreSQL computes norms for 32 vectors;
  maximum absolute error from one must be at most `1e-5`.
- Runtime output will use the suite step `artifact_dir` paths below. Cluster
  state uses `/home/peter/.ecaz/clusters/task167-exact-20260822-*`, outside the
  repository and outside Cargo `target/`, and will be removed after cited
  artifacts are captured.
- Runtime matrix status: pending; no measurement result is claimed by this
  checkpoint packet yet.
