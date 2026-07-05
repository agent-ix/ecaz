# Task 70 / Packet 002: Latency Session GUC Support

## Code Under Review

- Commit: `19c4a2126ea4dacd39ba38e6a833545ccaa11ba1`
- Scope: extends `ecaz bench latency` and `ecaz bench suite` latency steps with `session_gucs`, allowing benchmark-owned session settings such as `ec_diskann.scan_profile_notice=on`.

## Why

Task 70 Phase 1 needs profile NOTICE capture under `ecaz bench suite`, not one-off SQL or shell wrappers. Packet 001 added the opt-in `ec_diskann.scan_profile_notice` extension GUC; this packet wires the benchmark runner so a suite config can enable that GUC on every latency worker connection.

## Implementation Notes

- `ecaz bench latency --session-guc name=value` can be repeated.
- The latency command validates dotted GUC names by validating each identifier segment.
- Every worker connection applies the extra GUCs after the sweep GUC / rerank GUC and before adaptive/IVF options and query execution.
- Suite latency steps accept `session_gucs: ["ec_diskann.scan_profile_notice=on"]` and expand them to repeated `--session-guc` arguments.
- This is generic runner plumbing; no Task 70 benchmark matrix is hard-coded.

## Validation

See `artifacts/manifest.md`.

Commands run:

- `cargo fmt --check`
- `cargo test -p ecaz-cli parse_session_gucs`
- `cargo test -p ecaz-cli expands_latency_with_cache_state_label`
- `cargo check -p ecaz-cli`

## Follow-Up

Create the Task 70 Phase 1 suite packet using the existing `profile-cross-engine-real10k` surface narrowed to `ec_diskann` L=64/L=200 and enabling `session_gucs: ["ec_diskann.scan_profile_notice=on"]` on the latency step. Capture the resulting NOTICE lines, latency table, recall table, and EXPLAIN artifacts under the packet.
