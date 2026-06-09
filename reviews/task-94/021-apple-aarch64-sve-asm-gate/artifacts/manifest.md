# Manifest: Task 94 Packet 021

- Task bucket: `reviews/task-94/021-apple-aarch64-sve-asm-gate/`
- Code checkpoint: `20856cfb4d69e9a554acbe47cb9e7c4c78ac2dbc`
- Timestamp: `2026-06-09T19:18:39Z`
- Lane: coder-1 LUT lane
- Host: local x86_64 Linux
- AWS: not used
- CI: no rerun requested; existing failed job logs were inspected only

## Artifacts

- `artifacts/ci-failure-source.md`
  - Existing PR check failure:
    - `pg18 / stable`: https://github.com/agent-ix/ecaz/actions/runs/27229478676/job/80405533632
  - Key failure: Mach-O assembler rejected `.hidden` / `.type` directives
    inside grouped-PQ SVE `global_asm!`.
- `artifacts/cargo-fmt-check.log`
  - Command: `cargo fmt --check`
  - Result: pass
- `artifacts/cargo-clippy-pg18-bench.log`
  - Command: `cargo clippy --all-targets --no-default-features --features pg18,bench -- -D warnings`
  - Result: pass, `Finished dev profile`
- `artifacts/cargo-test-grouped-pq-lib.log`
  - Command: `cargo test grouped_pq --lib`
  - Result: pass, `35 passed; 0 failed; 2018 filtered out`
- `artifacts/cargo-check-aarch64-linux-pg18.log`
  - Command: `cargo check --target aarch64-unknown-linux-gnu --no-default-features --features pg18`
  - Result: blocked before crate validation by missing local cross C compiler:
    `failed to find tool "aarch64-linux-gnu-gcc"`.

## Notes

The local host does not have `aarch64-apple-darwin` installed and lacks the
Linux aarch64 C cross-compiler needed by C dependencies. Platform compile proof
for this packet is therefore expected from the next automatic PR matrix run.
This packet does not start that run manually.
