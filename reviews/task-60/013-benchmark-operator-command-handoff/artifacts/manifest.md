# Task 60 Benchmark Operator Command Handoff Artifacts

- Head SHA: `6dbf8bb61033404603a709dbaa0276a8e4c9acbd`
- Task bucket: `reviews/task-60/013-benchmark-operator-command-handoff/`
- Timestamp: 2026-05-26
- Lane: benchmark host handoff
- Fixture: Task 60 suite audit
- Storage formats: `pq_fastscan`, `rabitq`
- Rerank mode: default DiskANN benchmark recall/latency flow
- Shared-table surface: no; suite uses one prefix per size and storage format

## Artifacts

### `suite-audit.log`

Command:

```sh
cargo run -p ecaz-cli -- bench suite audit --config benchmarks/task60-diskann-rabitq-format/suite.json
```

Key result:

```text
[suite:task60-diskann-rabitq-format] audit passed: 24 steps
```

The benchmark-host manifest now documents the installed operator command
surface, `ecaz bench suite ...`; this local audit used `cargo run` because the
installed binary is absent in this sandbox.
