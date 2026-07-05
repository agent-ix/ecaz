# Artifact Manifest: 1036 AWS Representative Performance Attempt

- head SHA at packet closeout: `2020771db`
- task bucket: `reviews/task-30/1036-spire-phase13e-aws-representative-performance`
- lane: AWS representative performance, Graviton, `us-west-2a`, `m7g.large`, 1 coordinator + 3 remotes
- fixture: `pass-representative-performance`
- storage format: `rabitq`
- rerank mode: suite-controlled; representative measurement not reached
- isolated/shared surface: SPIRE representative coordinator plus remote shard surfaces

## Artifacts

### `run-representative-performance-pass-rerun-after-prepare-path-fix.log`

- command: `scripts/spire-aws/run-representative-performance-pass.sh --artifact-dir reviews/task-30/1036-spire-phase13e-aws-representative-performance/artifacts --reuse-artifact-dir --execute`
- result: failed during representative coordinator SPIRE build before latency/recall measurement
- key result: oversized manifest/object page payload failure surfaced by real `ec_real_100k`

### `spire-large-manifest-blob-pg18.log`

- command: `scripts/run_spire_large_manifest_blob_pg18.sh --artifact-dir reviews/task-30/1036-spire-phase13e-aws-representative-performance/artifacts --run-id manifest-local-20260527b --port 39820`
- result: local PG18 regression passed
- key result: `spire_large_manifest_blob_pg18_pass object_count=257 placement_count=257 indexed_rows=10`

### `run-representative-performance-pass-rerun-after-manifest-chain.log`

- command: `scripts/spire-aws/run-representative-performance-pass.sh --artifact-dir reviews/task-30/1036-spire-phase13e-aws-representative-performance/artifacts --reuse-artifact-dir --execute`
- result: failed before SPIRE build/query during representative coordinator corpus load
- key result: `COPY finish failed for ec_spire_aws_repr_1m_corpus`; teardown completed cleanly

### `coordinator-load-representative.log`

- command: emitted by the representative load step
- result: node reached `ec_real_100k` manifest/corpus inspection, then PostgreSQL connection closed during `COPY finish`

