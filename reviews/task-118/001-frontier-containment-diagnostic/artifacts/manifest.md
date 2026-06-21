# Artifact Manifest

Task bucket: `reviews/task-118/001-frontier-containment-diagnostic/`
Head SHA: `a2b1654e0fea655dc28e6f4ddb740529786139d1`
Timestamp: `2026-06-21T14:32:10Z`

## Artifacts

### `cargo-check-pg18-pgtest.log`

- Lane: local compile validation
- Fixture: none
- Storage format: HNSW diagnostic code path only
- Rerank mode: not applicable
- Isolated one-index-per-table surface: not applicable
- Command:

```text
cargo check --no-default-features --features pg18,pg_test
```

- Key result:

```text
Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.16s
```
