# Task 167 packet 041 artifacts

- Task bucket: `reviews/task-167/`.
- Packet: `041-exact-recall-gate`.
- Code checkpoint: `f83110078ec287060c1d2f714a17835084b3bd6c`.
- Exact runtime head (code plus this committed packet):
  `aebafec24b0b5d6734f4908f41248bff52128d5c`.
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
- Exact-runtime CLI rebuild command: the same release build command after
  packet commit. Result: passed in `build-cli-runtime.log` (SHA-256
  `c811cc67e9cc0ad5c491f7b63e0516fc5e5c24a117da4025d32b636d40d13172`).
- Exact-runtime CLI SHA-256:
  `f8ab415361968c0252ab0ad112cbddeadd635ecc06fee4298d2a6ca70ecca4e0`;
  the embedded runner SHA is the exact runtime head above.
- Production PG18 extension install command:
  `env CARGO_TARGET_DIR=/home/peter/.cargo-target cargo pgrx install --release --pg-config /home/peter/.pgrx/18.3/pgrx-install/bin/pg_config --features pg18 --no-default-features`.
- Extension install result: passed in `install-extension.log` (SHA-256
  `5fef533b378d6fd2d2b993e5a20cee518effcd9f5a64c53245f29c346113ea78`).
- Installed PG18 `ecaz.so` SHA-256:
  `1d4b39ab22bbb3c2f579e7fa9b385681c00f03a94a68c4a97d9a82e207804120`.
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
- Exact-runtime suite audit repeated after the CLI rebuild and extension
  install: passed, 3 steps, in `suite-audit-runtime.log` (same SHA-256
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
- Runtime suite command:
  `/home/peter/.cargo-target/release/ecaz bench suite run --config reviews/task-167/041-exact-recall-gate/artifacts/task167-exact-recall-suite.json --log-file reviews/task-167/041-exact-recall-gate/artifacts/suite-run.log`.
- 10k runtime provenance: release PG18 extension at
  `aebafec24b0b5d6734f4908f41248bff52128d5c`, three unanimous nodes, no debug
  override.
- PostgreSQL normalization preflight: 32 samples, maximum absolute norm error
  `0.000000017`, tolerance `0.000010000`, pass.
- Corrected 10k inserted-neighborhood result: 48 queries, 480 truth slots, 266
  distinct truth keys, 214 duplicate slots; physical `0.805382`, fresh
  `0.954985`, delta `-0.149603`, fail.
- Corrected 10k held-out result: 152 queries, 1,520 distinct truth keys, zero
  duplicate slots; physical `0.973684`, fresh `0.977632`, delta `-0.003947`,
  fail.
- Suite disposition: `physical-10k` failed with exit 1 after 599,805 ms;
  `physical-50k` and `physical-100k` remain pending/unrun because the suite was
  intentionally invoked without `--continue-on-error`.
- Result summary: `cited-results.log` (SHA-256
  `0e373ffc3e4b58c5ad2724ee6d445b528483648beca5ec3065505972967f1d36`).
  Raw sources: `final-suite/suite-manifest.json` (SHA-256
  `4c0b91296e791403e4a1d4391ca5f72f4765d4601445d24ff60b1f7f88056799`),
  `final-suite/physical-10k/distann-local-multinode.log` (SHA-256
  `41b6f9baad2331b976f52b2367b1e9249b2da9b7fdfa659f7b9c705fccd2085f`),
  and `suite-run.log` (SHA-256
  `1d523a21f2bddb1001107cf24e0cd65ea81ec5803134a6df796cf36c43b045f8`).
- This failed result re-derives the old `0.541667` disposition using the
  reviewer-requested exact instrument and proves a real incremental graph
  quality loss. It does not close Task 167.
