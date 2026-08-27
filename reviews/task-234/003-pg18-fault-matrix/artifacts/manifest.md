# Task 234 packet 003 artifact manifest

- Head SHA: `0f0fef9413fdd074a3f0016d1825fabf88896d72`
- Installed extension SHA: `c55f52b9358831c3a98c9b0c54661077de6b796b`
  - The two later commits (`945b39ad5`, `0f0fef941`) modify only
    `crates/ecaz-cli/src/commands/dev/distann_multicluster.rs`; extension
    sources are byte-identical to the installed SHA.
- Task bucket / packet: `reviews/task-234/003-pg18-fault-matrix/`
- Lane: PG18 four-owner physical multicluster
- Timestamp: 2026-08-24 PDT (America/Los_Angeles)
- Fixture: synthetic 2,000 rows, 16 dimensions, graph degree 32, four physical
  owners, one index per owner, coordinator is owner 1
- Extension: release profile, features
  `distann-head-attribution-benchmark,pg-test,pg18`; the attribution feature is
  an intentional dependency of `pg_test`; preflight used
  `--allow-debug-extension`
- CLI: debug diagnostic binary at head SHA; extension under test remained a
  release build
- Storage / rerank: unchanged; this packet validates read transport faults
- Run directory: `/home/peter/.ecaz/clusters/task234-read-rpc-fault-matrix`
  under the normal external cluster root; removed after evidence capture
- Corpus/query/truth data: none committed

## Commands

- Extension install:
  `cargo pgrx install --release --pg-config /home/peter/.pgrx/18.3/pgrx-install/bin/pg_config --no-default-features --features 'pg18 pg_test'`
- Focused CLI compile:
  `cargo check -p ecaz-cli`
- Diagnostic CLI build:
  `cargo build -p ecaz-cli`
- Matrix:
  `/home/peter/.cargo-target/debug/ecaz dev distann-multicluster local-multinode-pg18 --run-dir /home/peter/.ecaz/clusters/task234-read-rpc-fault-matrix --artifact-dir reviews/task-234/003-pg18-fault-matrix/artifacts/runtime --nodes 4 --rows 2000 --dim 16 --graph-degree 32 --base-port 43820 --read-rpc-fault-matrix --allow-debug-extension --skip-fault-drills`

## Artifacts and cited results

- `multicluster-console.log`
  - SHA-256:
    `0143e6f4c3c15da3a24f26d6bfce103d7ee2f6c52c06f9c51d5641371aeb67c0`.
  - Command exit 0.
  - Release preflight passed on all four nodes at extension SHA `c55f52b93`.
  - Ready and Published topology each covered exactly 2,000 records/rows with
    zero non-owned records and zero orphans.
  - Ordinary physical serving passed with 10 rows before fault injection.
  - Final line: `Task 234 read RPC fault matrix PASS cells=25`.
- `runtime/task234-read-rpc-fault-matrix.log`
  - SHA-256:
    `fd97d5994d15182e17b045fd4e603f468dc87d5e1c44e83bd1182676fc0329e1`.
  - Source of truth for all 25 cells: five RPCs by remote statement timeout,
    local query cancel, local statement timeout, remote backend termination,
    and connection reset.
  - Every cell has `pass=true`, `remote_work_drained=true`, and a positive
    `retry_rows` value.
  - Maximum failure elapsed time is 628 ms (connection reset), below the
    documented 5,000 ms reset/termination tolerance; every other class is at
    or below 507 ms, below its 2,000 ms tolerance.
  - The three fan-out remote-timeout cells each record
    `batch_total=3 batch_successes=2 batch_failures=1`, proving a sibling
    succeeded before the outward result was normalized fail-closed.
  - Remote statement timeout retains safe pool state; local cancel/timeout
    clears it; termination/reset evicts only the ambiguous target in fan-out
    calls and the sole connection in single calls.
- `runtime/node{1,2,3,4}-postgres.log`
  - Owner-side evidence for statement timeout, cancel, backend termination,
    immediate reset/recovery, and final clean shutdown.
- `cargo-pgrx-install-pg18-pg-test.log`
  - Release diagnostic extension install succeeded.
- `cargo-build-ecaz-cli-debug.log`
  - Final diagnostic CLI build succeeded with one pre-existing unrelated
    `LoadedDistributedPlacementConfig.path` dead-code warning.

Failed-attempt console/runtime exhaust was intentionally pruned. Before the
final exit-0 run, the harness needed two narrow corrections (qualifying
pg_test functions under schema `tests`, and refreshing the intentionally tiny
remote timeout before the clean import retry). One intervening fresh fixture
also hit a PostgreSQL `SubTransGetTopmostTransaction` assertion during the
ordinary pre-matrix serving smoke; no Task 234 fault cell ran in that attempt,
and both the preceding and final fresh fixtures passed the same serving smoke.
This packet does not use that aborted attempt as evidence.
