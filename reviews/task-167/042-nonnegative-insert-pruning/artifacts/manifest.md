# Task 167 packet 042 artifacts

- Task bucket: `reviews/task-167/`.
- Packet: `042-nonnegative-insert-pruning`.
- Code checkpoint: `a001bf7e6`.
- Trigger evidence: packet 041 corrected 10k exact-ground-truth failure.
- Root cause: incremental insert used negative inner-product distance in an
  alpha-prune algorithm whose distance contract is nonnegative; batch build
  uses shared `max(0, 1 - inner_product)`.
- Change isolation: `src/am/ec_distann/insert.rs`; one distance helper call and
  focused tests. No reloption, search-width, threshold, storage, or wire-format
  change.
- Validation command:
  `env CARGO_TARGET_DIR=/home/peter/.cargo-target cargo test -p ecaz am::ec_distann::insert::tests --lib --no-default-features --features pg18 --quiet`.
- Validation result: 10 passed, 0 failed, 2,566 filtered in
  `insert-validation-test.log` (SHA-256
  `f3e645ef84e508f75a9f497ed793429705f8511a4ea47efb12dfb4a8c95b775d`).
- Rerun suite config: `task167-distance-fix-suite.json` (SHA-256
  `d9958daab1c4af214edb68d8e885570a3ad5328bee6b60e1ce83a6e6ed1a9194`).
- Suite audit command:
  `/home/peter/.cargo-target/release/ecaz bench suite audit --config reviews/task-167/042-nonnegative-insert-pruning/artifacts/task167-distance-fix-suite.json --log-file reviews/task-167/042-nonnegative-insert-pruning/artifacts/suite-audit.log`.
- Suite audit result: passed, 3 steps, in `suite-audit.log` (SHA-256
  `0561a14ffd6fe3b83b6f075d028deb327d325ee2c65514e92f577a7e3faf5db3`).
- Matrix: production PG18, three owners, real 10k/50k/100k, 48 inserted plus
  152 held-out exact-truth queries at every scale, ordinary recall/latency,
  insert throughput/work, concurrency, and storage.
- Cluster state uses `/home/peter/.ecaz/clusters/task167-distance-20260822-*`,
  outside the repository and Cargo `target/`; each directory will be removed
  after cited artifacts are captured.
- Runtime status: pending; no performance or quality result is claimed yet.
