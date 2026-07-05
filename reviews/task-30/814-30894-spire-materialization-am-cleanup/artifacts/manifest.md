# Artifacts Manifest

Packet: `30894-spire-materialization-am-cleanup`

Head SHA: `74f511d4`

## Artifacts

- `cargo-test-custom-scan-lib.log`
  - head SHA: `74f511d4`
  - lane: Rust + PG18 pg_test filtered custom scan lane
  - command: `cargo test custom_scan --lib`
  - timestamp: `2026-05-11T22:56:06-07:00`
  - isolated/shared surface: shared extension catalog; CustomScan unit and
    PG18 fixtures
  - key result: `test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 1662 filtered out`

- `cargo-test-remote-search-final-contract-lib.log`
  - head SHA: `74f511d4`
  - lane: Rust + PG18 pg_test filtered final contract lane
  - command: `cargo test remote_search_final_contract --lib`
  - timestamp: `2026-05-11T22:56:06-07:00`
  - isolated/shared surface: shared extension catalog; remote search contract
    SQL
  - key result: `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1675 filtered out`

- `cargo-test-production-fault-matrix-lib.log`
  - head SHA: `74f511d4`
  - lane: Rust + PG18 pg_test filtered production fault matrix lane
  - command: `cargo test production_fault_matrix --lib`
  - timestamp: `2026-05-11T22:56:06-07:00`
  - isolated/shared surface: shared extension catalog; production fault matrix
  - key result: `test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 1674 filtered out`

- `cargo-test-phase7-policy-contracts-lib.log`
  - head SHA: `74f511d4`
  - lane: Rust + PG18 pg_test filtered operator contract lane
  - command: `cargo test phase7_policy_contracts --lib`
  - timestamp: `2026-05-11T22:56:06-07:00`
  - isolated/shared surface: shared extension catalog; operator entrypoint
    contracts
  - key result: `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1675 filtered out`

- `cargo-test-production-scan-am-lib.log`
  - head SHA: `74f511d4`
  - lane: Rust unit tests for AM delivery classification
  - command: `cargo test production_scan_am --lib`
  - timestamp: `2026-05-11T22:56:06-07:00`
  - isolated/shared surface: not applicable
  - key result: `test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 1669 filtered out`

- `cargo-test-production-scan-result-stream-am-outputs-lib.log`
  - head SHA: `74f511d4`
  - lane: Rust unit tests for AM output conversion
  - command: `cargo test production_scan_result_stream_am_outputs --lib`
  - timestamp: `2026-05-11T22:56:06-07:00`
  - isolated/shared surface: not applicable
  - key result: `test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 1674 filtered out`

- `cargo-test-local-heap-delivery-gate-lib.log`
  - head SHA: `74f511d4`
  - lane: Rust unit tests for local heap delivery gate
  - command: `cargo test local_heap_delivery_gate --lib`
  - timestamp: `2026-05-11T22:56:06-07:00`
  - isolated/shared surface: not applicable
  - key result: `test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 1674 filtered out`

- `cargo-fmt-check.log`
  - head SHA: `74f511d4`
  - lane: formatting
  - command: `cargo fmt --check`
  - timestamp: `2026-05-11T22:56:06-07:00`
  - isolated/shared surface: not applicable
  - key result: exited 0; stable rustfmt emitted the existing warnings about
    unstable `imports_granularity` and `group_imports` settings.

- `git-diff-check.log`
  - head SHA: `74f511d4`
  - lane: whitespace check
  - command: `git diff --check 74f511d4^ 74f511d4`
  - timestamp: `2026-05-11T22:56:06-07:00`
  - isolated/shared surface: not applicable
  - key result: exited 0 with no whitespace errors.
