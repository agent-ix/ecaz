# Task 167 packet 043 artifacts

- Task bucket: `reviews/task-167/`.
- Packet: `043-exact-recall-disposition`.
- Code checkpoint: `0bce21c05`.
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
- Run directories are under `/home/peter/.ecaz/clusters/`, outside the repo
  and Cargo target, and will be removed after cited artifacts are committed and
  pushed.
- Runtime status: pending.
