# Packet 035 Artifacts Manifest

| Artifact | Source | Command | Key results |
| --- | --- | --- | --- |
| `backing-page-emulator-focused-tests.log` | careful crate lib tests | `cargo test --manifest-path hardening/careful/Cargo.toml --lib` | `test result: ok. 472 passed; 0 failed` |
| `coverage/summary.txt` | `make coverage` | `make coverage COVERAGE_OUTPUT_DIR=reviews/task-39/035-backing-page-emulator/artifacts/coverage` | `am/ec_spire/page.rs ... 81.12%`, `am/ec_spire/storage/relation_store.rs ... 27.20%` |
| `coverage/coverage.json` | same `make coverage` run | (auto-emitted) | machine-readable summary |
| `coverage/careful-coverage.json` | same `make coverage` run | (auto-emitted) | per-file detail used by delta check |
| `coverage-delta-check.log` | baseline delta gate | `scripts/check_coverage_delta.sh reviews/task-39/035-backing-page-emulator/artifacts/coverage/summary.txt fixtures/quality/coverage-baseline.tsv reviews/task-39/035-backing-page-emulator/artifacts/changed-files.txt` | `coverage ok: am/ec_spire/page.rs actual=81.12 baseline=81.12`, `coverage ok: am/ec_spire/storage/relation_store.rs actual=27.20 baseline=27.20` |
| `coverage-baseline-check.log` | baseline completeness gate | `bash scripts/check_coverage_baseline_complete.sh fixtures/quality/coverage-baseline.tsv` | `coverage baseline complete for 40 critical paths` |
| `changed-files.txt` | hand-written | — | source paths whose baseline this packet ratchets |

Provenance:

- Head SHA at packet creation: see `git rev-parse HEAD` on
  `task39-continuation-20260519` at the time of the packet commit.
- Task bucket: `reviews/task-39/`.
- Packet path: `reviews/task-39/035-backing-page-emulator/`.
- Lane / surface: shadow careful crate (`hardening/careful/`), targeting
  `am/ec_spire/page.rs` and `am/ec_spire/storage/relation_store.rs`.
  Storage format / rerank mode are not applicable to this packet (no
  index lane is exercised).
- Surface: isolated unit tests inside the careful crate (no PostgreSQL
  socket, no live pgrx); each test re-seeds the emulator via
  `pg_sys::reset_counters()` so per-test state is independent.
