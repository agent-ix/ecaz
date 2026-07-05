# Manifest: Task 94 Packet 022

- Task bucket: `reviews/task-94/022-apple-aarch64-sve-helper-gate/`
- Code checkpoint: `8dabe603f35efa49055eb5d75ecd6fbffb77c298`
- Timestamp: `2026-06-09T19:26:16Z`
- Lane: coder-1 LUT lane
- Host: local x86_64 Linux
- AWS: not used
- CI: no rerun requested; existing failed job logs were inspected only

## Artifacts

- `artifacts/ci-failure-source.md`
  - Existing PR check failure:
    - `pg18 / stable`: https://github.com/agent-ix/ecaz/actions/runs/27230037274/job/80407424509
  - Key failure: `centroid_index` dead code on Apple aarch64 after the SVE impl
    was gated off that platform.
- `artifacts/cargo-fmt-check.log`
  - Command: `cargo fmt --check`
  - Result: pass
- `artifacts/cargo-clippy-pg18-bench.log`
  - Command: `cargo clippy --all-targets --no-default-features --features pg18,bench -- -D warnings`
  - Result: pass, `Finished dev profile`
- `artifacts/cargo-test-grouped-pq-lib.log`
  - Command: `cargo test grouped_pq --lib`
  - Result: pass, `35 passed; 0 failed; 2018 filtered out`

## Notes

The local host cannot compile the Apple aarch64 target. Platform proof for this
cfg alignment is expected from the next automatic PR matrix run; this packet does
not start that run manually.
