# Artifacts Manifest: 30719 SPIRE Nprobe Parsed Options

Head SHA: `b736da35be67ce9b71a25d01441dd9d09ac0c644`
Packet: `review/30719-spire-nprobe-parsed-options`
Timestamp: `2026-05-10T01:23:00-07:00`

## Artifacts

| Artifact | Lane | Fixture / Surface | Storage Format | Rerank Mode | Isolated Surface | Command | Key Result |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `cargo-check-pg18.log` | PG18 compile | extension compile | n/a | n/a | n/a | `cargo check --no-default-features --features pg18` | `Finished dev profile ... target(s) in 0.17s` |
| `cargo-fmt-check.log` | format | repository formatting | n/a | n/a | n/a | `cargo fmt --check` | command exited `0`; stable rustfmt emitted existing unstable-option warnings |
| `cargo-test-nprobe-per-level.log` | Rust unit | reloption parser | n/a | n/a | n/a | `cargo test --no-default-features --features pg18 nprobe_per_level` | `nprobe_per_level_reloption_parses_upper_level_values ... ok`; `1 passed; 0 failed` |
| `cargo-test-per-level-nprobe-policy.log` | Rust unit | recursive per-level nprobe policy | n/a | n/a | n/a | `cargo test --no-default-features --features pg18 per_level_nprobe` | `3 passed; 0 failed` |
| `cargo-test-routing-diagnostics-single-thread.log` | Rust unit | recursive routing diagnostics | n/a | n/a | n/a | `cargo test --no-default-features --features pg18 collect_scan_routing_diagnostics -- --test-threads=1` | `2 passed; 0 failed` |
| `git-diff-check.log` | diff hygiene | working diff | n/a | n/a | n/a | `git diff --check` | command exited `0` |
