# Manifest

- Task bucket: `reviews/task-40/`
- Packet: `reviews/task-40/001-execution-plan/`
- Head SHA: `a6a428b081bdb1fa9f00d51d0ceb18165a108204`
- Timestamp: 2026-05-18 America/Los_Angeles
- Lane: planning / code research
- Fixture: local repository inspection
- Storage format: not applicable
- Rerank mode: not applicable
- Shared-table versus isolated one-index-per-table: not applicable

## Commands

- `sed -n '1,220p' plan/tasks/40-*.md`
- `sed -n '1,220p' plan/tasks/README.md`
- `sed -n '1,180p' docs/hardening-governance.md`
- `sed -n '1,220p' reviews/README.md`
- `find hardening -maxdepth 3 -type f -print`
- `rg -n "Atomic|Mutex|RwLock|Once|LazyLock|OnceLock|Arc|thread|spawn|parallel|DSM|dsm|slot|claim|release" src/am hardening crates --glob '*.rs'`
- `rg -n "tokio|async|await|Tcp|libpq|remote|transport|partition|leader|retry|backoff|latency|pipeline" src/am/ec_spire crates docs --glob '*.rs' --glob '*.md'`
- `sed -n '1,620p' src/am/common/parallel.rs`
- `sed -n '1,1905p' src/am/ec_hnsw/build_parallel.rs`
- `sed -n '1,100p' scripts/hardening_validate.sh`
- `bash scripts/hardening.sh loom-real`
- `bash scripts/hardening_validate.sh`
- `cargo test --lib parallel_scan --no-run`

## Key Result Lines

- `docs/hardening-governance.md` says Task 34 `hardening/loom` and
  `hardening/shuttle` were removed because they were synthetic and can return
  only with real imports from `src/`.
- `src/am/common/parallel.rs` contains the production worker-slot claim,
  release, publish, and rescan protocol using `AtomicU32`.
- `src/am/ec_hnsw/build_parallel.rs` contains the concurrent DSM node
  `UNINSERTED -> INSERTING -> READY` protocol guarded by PG `LWLock`.
- SPIRE remote candidate dispatch uses Tokio/libpq surfaces, so deterministic
  network simulation is appropriate but distinct from Loom.
- `loom-real.log`: `test result: ok. 4 passed; 0 failed`.
- `parallel-scan-no-run.log`: `Finished test profile` and emitted
  `Executable unittests src/lib.rs`.

## Artifacts

- `loom-real.log`: final passing Loom run.
- `hardening-validate.log`: hardening lane validation.
- `parallel-scan-no-run.log`: production wrapper compile check.
