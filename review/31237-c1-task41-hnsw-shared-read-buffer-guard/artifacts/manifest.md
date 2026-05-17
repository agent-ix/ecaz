# Artifact Manifest

- Packet: `31237-c1-task41-hnsw-shared-read-buffer-guard`
- Head SHA: `390b18b4`
- Timestamp: `2026-05-17T22:09:45Z`
- Surface: static validation only; HNSW shared read buffer lock ownership changed
- Storage/rerank mode: HNSW shared metadata/data page reads and PG18 read-stream
  tuple counting
- Shared-table vs isolated-table: not applicable

## Artifacts

- `validation.md`
  - Command summary and key result lines for this packet.
  - Baseline movement cited by `request.md`: `3983 -> 3966`.
