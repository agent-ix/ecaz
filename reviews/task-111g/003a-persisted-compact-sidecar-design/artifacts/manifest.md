# Manifest — Task 111g Packet 003a (persisted compact sidecar design checkpoint)

- **Head SHA:** `85443b51d` (no code change in this packet; design + baseline check only)
- **Task bucket / packet:** `reviews/task-111g/003a-persisted-compact-sidecar-design/`
- **Lane:** N/A (design checkpoint; no benchmark)
- **Surface:** N/A (no DB run)

## Artifacts

| file | what | command | timestamp |
|------|------|---------|-----------|
| `cargo-check-baseline.log` | branch tip builds before 003b | `cargo check --no-default-features --features pg18` | 2026-06-18 |

## Key result lines cited by request.md

- `cargo-check-baseline.log`: `Finished \`dev\` profile … target(s)` — base SHA
  `85443b51d` compiles under pg18; 003b starts from a green tree.

## Design summary (full rationale in request.md)

- Recommend **Option B**: persisted compact rerank sidecar keyed by heap TID,
  new index page tag `0x2A`, metadata head pointer at a spare metadata offset.
  f16 = `dims*2` bytes, rabitq4 = rabitq4 payload_len; f32 keeps the heap source
  (bit-identical, no sidecar).
- Rejected **Option A** (rerank_tid-through-candidate): would restructure the
  dense coalesced SoA hot path, violating "do not change coalescing".
- Win evidence for 003b: new explain counter `stats_rerank_source_bytes_read`;
  pg_test asserts f16/rabitq4 read fewer rerank source bytes than f32 on the same
  corpus — proves the byte reduction without `ecaz bench suite`.

## Note on the existing on-disk hook

`EC_IVF_POSTING_RERANK_TID_OFFSET` / `IvfDensePostingBlockRef::rerank_tid_bytes`
already exist and are written always-INVALID today (`build.rs:680`, `:705`). The
sidecar design does not require an on-disk format break; under Option B the
per-posting `rerank_tid` slot remains unused (reserved) and rerank uses the
heap-TID-keyed `0x2A` sidecar instead.
