# Review Request: Concurrent DSM Default Switch

## Summary

Please review commit `5aedea0`, which switches eligible parallel `ec_hnsw`
builds to concurrent DSM graph assembly by default.

Changes:

- `ec_hnsw.enable_parallel_build_concurrent_dsm` now defaults to `on`
- the GUC remains available as a diagnostic fallback
- the PG18 concurrent DSM smoke test now verifies default behavior without an explicit opt-in
- a new PG18 smoke test verifies `SET ec_hnsw.enable_parallel_build_concurrent_dsm = off` suppresses graph worker launch

## Validation

- `cargo pgrx test pg18 test_pg18_parallel_index_build_concurrent_dsm_graph_default`
- `cargo pgrx test pg18 test_pg18_parallel_index_build_concurrent_dsm_can_be_disabled`
- `cargo test`
- `cargo pgrx test pg18`
- `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
- `git diff --check`

`cargo fmt --check` still reports pre-existing unrelated formatting drift in `crates/ecaz-cli/src/commands/quant/feasibility.rs` and `src/quant/rabitq.rs`.

## Measurement Context

No new performance run is attached to this packet. The default switch is based on the Phase 3 packet chain:

- packet 658: real 50k recall parity at recall@10 `0.91` / `0.91`
- packet 659: serial source-scored build `30:15.962`
- packet 660: post-AVX concurrent DSM source-scored build `03:27.595`
- packet 663: current best concurrent DSM source-scored build `03:17.371`, recall@10 `0.91`

## Notes

The old serial leader graph path is still reachable via the GUC fallback. Removing superseded shm_mq ingestion code should be handled as a separate cleanup slice so rollback remains straightforward if reviewers want the default switch isolated.
