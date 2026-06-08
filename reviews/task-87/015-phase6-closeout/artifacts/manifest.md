# Task 87 Packet 015 Artifact Manifest

Head SHA: `9e4d2ad3642b63eec543d9710ca521d1b8f82787`

Task bucket: `reviews/task-87/`

Packet path: `reviews/task-87/015-phase6-closeout/`

Timestamp: 2026-06-08

## Artifacts

### `aggregate-matrix.md`

- Lane: Task 87 Phase 6 aggregate closeout.
- Fixture: DBPedia real10k, real50k, real100k.
- Access methods: HNSW, IVF, SPIRE.
- Storage format / rerank mode: existing per-packet surfaces from packets
  012, 013, and 014; see those packet manifests for each exact prefix,
  storage format, and rerank setting.
- Isolation: one index surface per AM/corpus; off/on cells reuse the same
  index and flip only a session GUC for route comparison.
- Command used: no new benchmark command; aggregation of checked-in
  `ecaz bench suite` outputs from packets 012, 013, and 014.
- Key result lines:
  - Recall is unchanged across off/on in all nine SPIRE/IVF/HNSW cells.
  - SPIRE p50 improves 36.9%, 28.6%, and 34.2% for real10k, real50k, and
    real100k respectively.
  - IVF improves at real10k and real100k; real50k RaBitQ is flat/slightly
    worse at p50/p95 and slightly faster at p99.
  - HNSW improves most at real50k p95/p99, but real100k p50 is effectively
    flat.
  - DiskANN remains represented by packet 005 Stop Condition and packet 009
    Task 91 handoff.

## Source Evidence

- `reviews/task-87/012-phase6-suite-prep/artifacts/manifest.md`
  - real10k suite outputs and SPIRE rerun outputs.
- `reviews/task-87/013-phase6-real50k-matrix/artifacts/manifest.md`
  - real50k suite outputs.
- `reviews/task-87/014-phase6-real100k-matrix/artifacts/manifest.md`
  - real100k suite outputs.
- `reviews/task-87/005-phase4-diskann-stop-condition/`
  - accepted DiskANN Task 87 Stop Condition evidence.
- `reviews/task-87/009-scope-walk-back-and-task-91-handoff/`
  - reviewer-approved Task 91 handoff and no-new-HNSW/DiskANN-QuantCodec
    operational guard.

No new tests or benchmarks were run for this closeout packet. It is an
aggregation and status-update packet over already checked-in suite evidence.
