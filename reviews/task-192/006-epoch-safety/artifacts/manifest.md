# Artifact manifest — Task 192 packet 006

- Head SHA: `1a52b08b5b2b679c46d0bc8b6bfe1ec8ac2f3a76`
- Task bucket / packet: `reviews/task-192/006-epoch-safety/`
- Primary target: PG18
- Storage / rerank / wire behavior: unchanged; feature-gated backend cache and test-only inspection surface only
- Benchmark state: none in this packet; packet 005 owns the isolated 100k A/B and the next packet will own the 10k/50k/100k suite

## Files and commands

| Artifact | SHA-256 | Command / result |
|---|---|---|
| `pg18-multi-epoch-cache.log` | `3a0675798413761ee24a60ef77356b773f033c5e100984f63fadf678229793e1` | `PGRX_PG_CONFIG_PATH=/home/peter/.pgrx/18.3/pgrx-install/bin/pg_config cargo pgrx test pg18 test_distann_multi_epoch_publish --no-default-features --features pg18`; 1 passed, 0 failed |
| `production-pg18-check.log` | `f197bcaeca8722d8049efd29da94584b10795281fe990164a691a27555477068` | `cargo check --no-default-features --features pg18`; pass |
| `attribution-pg18-check.log` | `f86c9d805ead36a72aa5c78c9273d94906437ea8f6a6173e085d6b7519c6bdc4` | `cargo check --no-default-features --features pg18,distann-head-attribution-benchmark`; pass |

The pgrx test log also records that its debug install wrote
`/home/peter/.pgrx/18.3/pgrx-install/lib/postgresql/ecaz.so`. Post-test size was
330,795,520 bytes, matching `target/debug/libecaz.so`; this is explicitly not a
benchmark binary.
