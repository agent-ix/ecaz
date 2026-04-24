# Review Request: Parallel Scan Locked Owned Read-Take

Current head: `7068834ae7e5da75bfcf64e2a8e1aa0edc079b39`

Scope:
- `src/am/common/parallel.rs`
- `src/am/ec_hnsw/scan.rs`

Problem:
- The scan path read owned-output readiness and then took owned output in two
  separate coordinator-lock critical sections.
- A stale `Ready` decision could become invalid before the take path ran,
  producing the fatal `ready state produced no owned output to take` path
  instead of re-evaluating blockers atomically.
- The risky seam mattered most when a better hidden foreign row could block an
  owner row between the readiness read and the output take.

What changed:
- Split the existing owned readiness and owned take logic into lock-held helper
  forms while preserving the original public wrappers.
- Added `take_parallel_scan_owned_ready_output_snapshot`, which reads readiness
  and takes owned output under one coordinator lock, returning `Empty`, `Ready`,
  or `Blocked`.
- Updated `try_take_parallel_scan_next_output` to use the combined state, and
  extracted the existing blocker handling into a helper so the blocked path is
  shared unchanged.
- Added a regression proving the combined path reports the better hidden
  foreign blocker and leaves the owner row staged instead of draining it.

Artifact:
- `artifacts/pg18-parallel-scan.log` captures the safe PG18 CLI preflight after
  the change.
- The log shows the ordered `ec_hnsw` candidate IDs match the serial IDs while
  the default plan remains a serial `Index Scan`; the control plan still proves
  PostgreSQL can launch workers for the fixture.

Validation:
- Passed:
  - `cargo test`
  - `cargo pgrx test pg18`
  - `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
  - `cargo pgrx install --release --pg-config /home/peter/.pgrx/18.3/pgrx-install/bin/pg_config --features 'pg18 pg_test' --no-default-features`
  - `cargo run -p ecaz-cli -- dev test pg18-parallel-scan`
  - `cargo test take_parallel_scan_owned_ready_output_snapshot_preserves_hidden_foreign_blocker --lib`
  - `cargo test try_take_parallel_scan_next_output --lib`
  - `cargo fmt --check`
  - `git diff --check`

Review focus:
- Whether the combined read-and-take helper is the right lock boundary for the
  owned-output seam.
- Whether preserving the old public wrappers keeps the coordinator API clear
  enough for existing unit coverage and future cleanup.
- Whether the hidden-foreign-blocker regression covers the stale-ready failure
  mode tightly enough before continuing into the remaining live parallel emit
  coordination work.
