# Task 60 Benchmark Host Handoff Artifacts

- Head SHA: `97fe4c5aa54fae9d7e684bb08cc8031902aa5d04`
- Task bucket: `reviews/task-60/008-benchmark-host-handoff/`
- Timestamp: 2026-05-25
- Lane: benchmark packet handoff
- Fixture: dry-run suite expansion only
- Storage formats: `pq_fastscan`, `rabitq`
- Rerank mode: default DiskANN benchmark recall/latency flow
- Shared-table surface: no; suite uses one prefix per size and storage format

## Artifacts

### `suite-audit.log`

Command:

```sh
cargo run -p ecaz-cli -- --log-file reviews/task-60/008-benchmark-host-handoff/artifacts/suite-audit.log bench suite audit --config benchmarks/task60-diskann-rabitq-format/suite.json
```

Key result:

```text
[suite:task60-diskann-rabitq-format] audit passed: 24 steps
```

### Benchmark Packet Artifacts

- `benchmarks/task60-diskann-rabitq-format/artifacts/suite-dry-run.log`
- `benchmarks/task60-diskann-rabitq-format/artifacts/suite-manifest.json`

The dry-run was generated from the manifest command that now includes a
packet-local `--log-file`.
