# Task 51 AWS IVF/RaBitQ Final Gate

- Status: prepared; AWS stack is currently down.
- Head SHA at packet creation: `cb21fb674f6b368e93611ba5d6c0f4dbf9b629e3`.
- Task bucket: `reviews/task-51/`.
- Benchmark packet: `benchmarks/task51-aws-ivf-rabitq-final-gate/`.
- Lane: AWS final gate, IVF and RaBitQ only.
- Planned instance shape: restored PostgreSQL snapshot on Graviton, preserving existing indexes.
- Planned snapshot source: `snap-091251b06d2da2df4`.
- Planned table/index prefix: `real_1m_ivf_rabitq1_rerank`.
- Planned access method/storage: `ec_ivf`, `storage_format=rabitq`.
- Planned reloptions from preserved snapshot: `quant_bits=1`, `rerank=heap_f32`, `rerank_width=50`.
- Planned suite config: `suite.json`.
- Isolation note: the AWS run reuses the isolated one-index-per-table IVF/RaBitQ surface already present in the preserved snapshot. No vchord or pgvectorscale steps are part of this packet.

## Preflight Artifacts

- `artifacts/cloud-status-10k-medium.log`: `10k-medium` profile was down, with retained snapshot `snap-091251b06d2da2df4`.
- `artifacts/cloud-status-1m.log`: `1m` profile was down during profile inventory.
- `artifacts/cloud-status-10k.log`: `10k` profile was down during profile inventory.
- `artifacts/cloud-status-10k-medium-second-check.log`: repeated `10k-medium` status check before AWS bring-up.
- `artifacts/cloud-up-dry-run-no-snapshot-vars.log`: dry-run evidence showing the current `cloud up --dry-run` path does not include the snapshot override vars, so final apply must explicitly preserve the snapshot/volume shape.
- `artifacts/suite-audit-local.log`: local suite audit passed for 5 steps.
- `artifacts/suite-dry-run-local.log`: local suite dry-run expanded only IVF/RaBitQ recall, latency, storage, and EXPLAIN steps.
- `artifacts/suite-dry-run-manifest.json`: dry-run manifest generated from `suite.json`.

## Planned Commands

```sh
target/release/ecaz bench suite audit \
  --config benchmarks/task51-aws-ivf-rabitq-final-gate/suite.json

target/release/ecaz bench suite run \
  --dry-run \
  --config benchmarks/task51-aws-ivf-rabitq-final-gate/suite.json \
  --manifest-output benchmarks/task51-aws-ivf-rabitq-final-gate/artifacts/suite-dry-run-manifest.json
```

AWS execution will be recorded after the restored database is verified to contain the preserved IVF/RaBitQ table and index.
