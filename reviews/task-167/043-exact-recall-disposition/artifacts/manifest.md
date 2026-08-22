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
  and Cargo target. They are per-scale isolated fixtures, never evidence, and
  will be removed after cited artifacts are committed and pushed.

## Production matrix run

- Head SHA at run start: `e5cbafb58596934bf21cc7ac601295f8b1f3e8cc`.
- Runtime CLI: `ddf5dbdce2bace098650678aef5b39713b1ebd9a/release`;
  runtime extension: `01da3574498fcd30cef6b29e14cf4ca7f3872326/release`.
- Command:
  `/home/peter/.cargo-target/release/ecaz bench suite run --config reviews/task-167/043-exact-recall-disposition/artifacts/task167-disposition-suite.json --log-file reviews/task-167/043-exact-recall-disposition/artifacts/suite-run.log`.
- Time: `2026-08-22T10:28:48-07:00` through
  `2026-08-22T13:17:00-07:00`.
- Result: all three steps succeeded with exit code 0. Step durations were
  `1319031 ms` (10k), `3213415 ms` (50k), and `5568101 ms` (100k).
- Isolation: each scale used its own three-node PG18 fixture, port range, run
  directory, corpus tables, physical index, single-node control index, and
  fresh comparison index. Each table had one index; variants shared only the
  owning scale's fixture so the insert A/B remained same-fixture attributable.
- Lane: physical distributed DistANN, real staged `ec_real_10k`,
  `ec_real_50k`, and `ec_real_100k`; rabitq stored neighbor codes; exact fp32
  truth; no rerank variant.
- Corpus data and truth caches are not committed. Corpus prefixes, query
  hashes, commands, runtime SHAs, and per-step topology are in the summaries,
  suite manifest, and structured results.

## Cited results

- Compact source of request values: `cited-results.log`, SHA-256
  `920f39123d4f181c39de9434811fd7276a82a578b3ced7ca6bed855829ffa358`.
- Exact fp32 recall, physical / fresh / delta:
  - 10k inserted `0.940600 / 0.954985 / -0.014385`; heldout
    `0.974342 / 0.977632 / -0.003289`.
  - 50k inserted `0.952257 / 0.940972 / +0.011285`; heldout
    `0.853289 / 0.879276 / -0.025987`.
  - 100k inserted `0.933160 / 0.936632 / -0.003472`; heldout
    `0.802632 / 0.808553 / -0.005921`.
- Ordinary distinct recall: `0.9990 / 0.9535 / 0.9280`; mean latency:
  `17.50 / 20.40 / 19.50 ms`; physical generation bytes:
  `242958336 / 1243553792 / 2498248704` at 10k/50k/100k.
- Same-fixture append-enabled/disabled throughput ratios:
  `0.975741 / 0.997529 / 0.993053`; each honestly reports `pass=false`.
- Concurrency, routed delete/vacuum, and topology gates pass at all scales.
  The 100k concurrency wave observed 23 natural 2PC retries.

## Durable artifacts

- `final-suite/suite-manifest.json`: canonical commands, configuration SHA,
  timing, step status, and exit codes; SHA-256
  `85daa3fbd55485cea4a187660f89b923ba8dda3c827e2c9f592f4fa45aed03ea`.
- `final-suite/results.jsonl`: structured metrics cited above; SHA-256
  `b6e50617c6fce002284e68051680af0b506536acdec8d1bd884eedcb152f22a2`.
- Per-scale `distann-multinode-summary.log`: primary raw summaries; SHA-256
  `677e88bbdcf9aebd7d1bbf3bce3b2eb133827bff7836efa1b7d7b06515f1f06b`
  (10k),
  `3551bb61c4d99bac48dc99df2006675a5b249311474873493ecf5d4fade9d50e`
  (50k), and
  `756aa60819f0b12c3520db3d2c8707fc78b6c6cef2a6e11b0212c8535d63ccb0`
  (100k).
- `final-suite/artifact-sha256.txt`: SHA-256 inventory for every generated
  suite artifact retained in the packet; inventory SHA-256
  `4c2c0178d26c55ccd15dbb41961eb1e51ebf7468a0155c6ef3153fd88a23065a`.
- `suite-run.log`: suite-level runner log; SHA-256
  `8509d8d348c1dbdd9e7f5f4cc281d39cb8b1f14a3ea1ee27a9e5ec1ccb48dc3a`.
- Per-scale raw fixture stdout, recall, latency, predictions, and head
  membership files are retained and inventoried. Node PostgreSQL operational
  logs were removed before commit; they are not decision-grade evidence.

Runtime status: full 10k/50k/100k matrix complete; outside reviewer
disposition pending.
