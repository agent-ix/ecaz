# Task 63 Packet 006 Artifact Manifest

- head SHA: `a9d4930bc0e5be1ab4115d474af5f96416176ea8`
- task bucket: `reviews/task-63/006-hnsw-rabitq-insert-vacuum`
- lane: HNSW RaBitQ insert and vacuum
- fixture: compile-only validation and targeted test compile
- storage format: `rabitq`
- rerank mode: cold scalar-quantized rerank payload
- isolated one-index-per-table surface: not applicable; no SQL benchmark or
  smoke run in this packet
- timestamp: 2026-05-26

## Commands

```text
cargo check -q --lib
cargo test -q --lib graph_storage_descriptor_uses_rabitq_code_len_for_v4_metadata --no-run
```

## Key Results

```text
cargo check -q --lib
```

completed successfully with no diagnostics.

```text
cargo test -q --lib graph_storage_descriptor_uses_rabitq_code_len_for_v4_metadata --no-run
```

completed successfully. It emitted existing warnings from test-only helpers:
unnecessary `unsafe` blocks in HNSW debug test macros and unused Hadamard test
helpers.

## Notes

PG18 SQL smoke coverage and benchmark-suite evidence remain follow-up Task 63
slices.
