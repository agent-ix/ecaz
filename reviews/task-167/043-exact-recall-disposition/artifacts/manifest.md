# Task 167 packet 043 artifacts

- Task bucket: `reviews/task-167/`.
- Packet: `043-exact-recall-disposition`.
- Code checkpoint: `0bce21c05`.
- Exact CLI runtime head (code plus initial packet):
  `ddf5dbdce2bace098650678aef5b39713b1ebd9a`.
- Trigger evidence: packet 042 exact-truth 10k values were valid but stopped
  the required matrix on an unsupported hard `0.002` verdict.
- Change isolation:
  `crates/ecaz-cli/src/commands/dev/distann_multicluster.rs`; exact-recall
  measurement labeling and process disposition only. No extension, graph,
  storage, reloption, query, truth, or workload change.
- Validation command:
  `env CARGO_TARGET_DIR=/home/peter/.cargo-target cargo test -p ecaz-cli task167_ --no-default-features --quiet`.
- Validation result: 9 passed, 0 failed in `task167-cli-tests.log` (SHA-256
  `4fc581e6d73512af851876b1a55a13dd5bc6bc8cf47aa6bce5795c93967d78fd`).
- Matrix config: `task167-disposition-suite.json`; production PG18, three
  owners, real 10k/50k/100k, 48 inserted-neighborhood plus 152 held-out exact
  fp32 truth queries, ordinary recall/latency, insert throughput/work,
  concurrency, and storage. SHA-256
  `10612eafe98409b33137b893735ff18ff99ce95974221abb6ff130c25cfc0676`.
- Suite audit command:
  `/home/peter/.cargo-target/release/ecaz bench suite audit --config reviews/task-167/043-exact-recall-disposition/artifacts/task167-disposition-suite.json --log-file reviews/task-167/043-exact-recall-disposition/artifacts/suite-audit.log`.
- Suite audit result: passed, 3 steps in `suite-audit.log` (SHA-256
  `e4bb53217dad093c26f6516e2741217560a2bb2448ea649bc208fdb35aa8e165`).
- Exact-head release CLI build command:
  `env CARGO_TARGET_DIR=/home/peter/.cargo-target cargo build -p ecaz-cli --release --no-default-features`.
- Release build result: passed; the cached capture is in `build-cli.log`
  (committed SHA-256
  `23034f86bdf0bd773376b2356b9f9a697f467754d90190e59f2c880de0500b04`),
  with one pre-existing dead-code warning at `commands/corpus/load.rs:190`.
- Release CLI SHA-256:
  `c0b12c38e048e11607f01c5028b5b0dc0225bd97a909381a462979dd6f771653`;
  embedded git SHA and profile are
  `ddf5dbdce2bace098650678aef5b39713b1ebd9a/release`.
- Installed PG18 extension remains the packet-042 pruning-fix runtime:
  embedded git SHA/profile
  `01da3574498fcd30cef6b29e14cf4ca7f3872326/release`, SHA-256
  `222a42b372f36ed92a8856f6305bb18e8beb9fda32af77284f0b14381faaeeae`.
  Packet 043 changes no extension code.
- Exact-runtime suite audit repeated after the CLI build: passed, 3 steps in
  `suite-audit-runtime.log` (SHA-256
  `e4bb53217dad093c26f6516e2741217560a2bb2448ea649bc208fdb35aa8e165`).
- Run directories are under `/home/peter/.ecaz/clusters/`, outside the repo
  and Cargo target, and will be removed after cited artifacts are committed and
  pushed.
- Runtime status: pending.
