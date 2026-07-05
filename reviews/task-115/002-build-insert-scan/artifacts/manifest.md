# Task 115 / 002 manifest

- Task bucket: `reviews/task-115/`
- Packet path: `reviews/task-115/002-build-insert-scan/`
- Head SHA: `15ba3e0f521b30f101599d930e1b7b98c0d3ae1b`
- Code commits: `7208d8226` (Phase 2), `00f2655c6` (Phase 3), `15ba3e0f5` (carry-forward test)
- Branch: `task-115-ivf-rabitq-residual-quantization`
- Phases: 2 (gated build/insert encoding) + 3 (scan integration + 113 recall-safety)
- Host: dev/review box (no `ecaz` binary, no staged corpora). pg18 pgrx tests +
  cargo unit tests + clippy only. No benchmark numbers (NFR-007; Phases 4/5
  deferred — configs shipped, not run).

## Artifacts

| file | what | command |
|------|------|---------|
| `phase23-residual-pg18-tests.log` | 5 residual pgrx tests, all passed | `cargo pgrx test pg18 --no-default-features --features "pg18 pg_test" test_ec_ivf_rabitq_residual` |
| `phase23-unit-tests.log` | 146 ec_ivf unit tests + 4 residual scalar tests | `cargo test --no-default-features --features pg18 --lib am::ec_ivf` / `... quant::rabitq::tests::residual` |
| `cargo-clippy.log` | clippy clean, all targets | `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings` |
| `task-115-residual-recall-per-probe.intel-local.json` | Phase 4 plain-vs-residual recall-per-probe sweep (DEFERRED to bench host) | `ecaz bench suite run --config <this> --artifact-dir <packet>/artifacts/bench-residual` |
| `task-115-residual-matched-recall-latency.intel-local.json` | Phase 5 matched-recall latency confirmation (DEFERRED; run only if recall improves) | `ecaz bench suite run --config <this> --artifact-dir <packet>/artifacts/bench-residual-matched` |

## Key result lines cited by request.md

```
test tests::pg_test_ec_ivf_rabitq_residual_coexists_with_plain ... ok
test tests::pg_test_ec_ivf_rabitq_residual_heap_f32_rerank_matches_plain ... ok
test tests::pg_test_ec_ivf_rabitq_residual_posting_bound_prune_equals_unpruned ... ok
test tests::pg_test_ec_ivf_rabitq_residual_build_equals_insert ... ok
test tests::pg_test_ec_ivf_rabitq_residual_insert_after_build ... ok
test result: ok. 5 passed; 0 failed; ...
test am::ec_ivf::page::tests::metadata_roundtrips_rabitq_residual_flag ... ok
test result: ok. 146 passed; 0 failed; ...   (ec_ivf unit)
```

## On-disk format note

- `MetadataPage.rabitq_residual` stored at byte 35 (between quant_bits at 34 and
  centroid_head at 36); previously-zero byte → plain (recall-safe default).
  Research project, no backward-compat requirement — residual is a clean new mode.
- Residual posting payload length == plain payload length (zero extra per-posting
  metadata; the centroid term is exact + per-list). Index size is residual-neutral.

## Non-standard bench configs (justification, per the standard-sweep convention)

Both shipped configs are NON-STANDARD vs the canonical lane sweep, justified in
each config's `comment` field: the plain-vs-residual arm is a **build-time
reloption** (`rabitq_residual=0|1`), not a session GUC, so the A/B requires two
separately-built indexes — the canonical config has no such arm. nprobe sweeps use
the registered ec_ivf `default_sweep` `[8,16,24,32,48,64]` verbatim. Matched-recall
config nprobe values are placeholders to be filled from Phase-4 results.

## Re-run (this box)

```
cargo pgrx test pg18 --no-default-features --features "pg18 pg_test" test_ec_ivf_rabitq_residual
cargo test --no-default-features --features pg18 --lib am::ec_ivf
cargo test --no-default-features --features pg18 --lib quant::rabitq::tests::residual
cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings
```

## Deferred to the bench host (Intel desktop)

- Stage `ec_real_100k` at `data/staged-current/`, then run both configs.
- Build the matched-recall comparison table; decide promote/iterate/abandon.
- Promotion (flipping the default to residual) is gated on a material
  recall-per-probe win with residual-neutral index size.
