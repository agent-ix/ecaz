# Review Request: HNSW RaBitQ Final Local Handoff

- task: `plan/tasks/63-hnsw-rabitq-storage-format.md`
- branch: `task/60-diskann-rabitq`
- packet: `reviews/task-63/016-hnsw-rabitq-final-local-handoff/`

## Summary

This packet updates the canonical Task 63 task file and benchmark manifest
after the final local closeout/audit packets. It does not change HNSW RaBitQ
code or run benchmarks.

## Touched Files

- `plan/tasks/63-hnsw-rabitq-storage-format.md`
  - notes that packets `011-*` through `015-*` now cover benchmark handoff,
    docs caveat, HNSW V4 RaBitQ fixture/upgrade matrix, reloption/spec text,
    and byte-LUT allocation audit.
- `benchmarks/task63-hnsw-rabitq-format/manifest.md`
  - records the current branch head for faster-host install:
    `f20e91c3494060ba64927bf9482112a3011438a0`;
  - preserves `36807d607606808717e0b645cde9b251d3fa2e23` as the minimum code
    source head for valid post-scorer measurements;
  - points future measurement agents at packets `011-*` through `015-*` for the
    local handoff trail.

## Validation

No tests or benchmarks were run. This is a metadata/handoff-only packet.
