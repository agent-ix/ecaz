# Manifest — Task 111g Packet 003b (persisted compact rerank sidecar)

- **Task bucket / packet:** `reviews/task-111g/003b-persisted-compact-sidecar/`
- **Branch:** `task-111g-coarse-rerank-representations`
- **Base SHA:** `0710f9f4c`
- **Code SHAs:** `3f9e990c0` (sidecar + metadata v3 + docs), `1ca3ae0f3` (vacuum
  sidecar maintenance + coarse_rerank vacuum payload_len fix)
- **Lane / host:** local PG18 dev (pgrx `cargo pgrx test pg18`); **no bench env**,
  no `ecaz bench suite`, no fabricated numbers (NFR-007). This is a code-review
  packet; the latency/recall suite remains the packet-002 Phase-3 gate on a
  provisioned lane.
- **Storage format under review:** `ec_ivf` `storage_format='coarse_rerank'`,
  `rerank_placement='index'`, `rerank_format ∈ {f16, rabitq4}`; compared against
  `rerank_placement='table'` / `rerank_format='f32'`.
- **Surfaces:** isolated one-index-per-table (each pg_test creates its own table
  + index).

## Artifacts

| File | What |
| --- | --- |
| `cargo-clippy.log` | `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings` |
| `cargo-test-integration.log` | `cargo test` for `size_of_assertions` + `on_disk_fixtures` + `upgrade_matrix` |
| `pgrx-test-pg18-111g.log` | `cargo pgrx test pg18` filtered to the 111g sidecar fixtures |

## Commands

```
cargo check --no-default-features --features pg18
cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings
cargo test --no-default-features --features pg18 \
  --test size_of_assertions --test on_disk_fixtures --test upgrade_matrix
cargo pgrx test pg18    # full suite; 111g fixtures cited below
```

## Key result lines

### clippy / check
- `cargo check` and `cargo clippy --all-targets ... -D warnings`: clean (0
  warnings).

### Integration tests
- `size_of_assertions`: 13 passed — pins `EC_IVF_INDEX_FORMAT_VERSION == 3`,
  `EC_IVF_METADATA_BYTES == 86`, `EC_IVF_METADATA_BYTES_V2 == 80`,
  `EC_IVF_METADATA_RERANK_SIDECAR_HEAD_OFFSET == 80`.
- `on_disk_fixtures`: 47 passed — incl. `ivf_metadata_v1_fixture_decodes`
  asserting the committed 80-byte v1 image decodes under the v3 reader with
  `rerank_sidecar_head == INVALID` (durable backward-read proof).
- `upgrade_matrix`: 2 passed — ivf writable set `{3}`, rows for `1`/`2`
  (read-only) and `3` (read+write).

### pg_test (PG18) — 111g sidecar fixtures
- `test_ec_ivf_index_placement_f16_rabitq4_admin_snapshot`: PASS
- `test_ec_ivf_index_placement_f16_matches_table_f16_ranking`: PASS
  (index-side f16 == table-side f16, bit-identical scores + same top-5).
- `test_ec_ivf_index_placement_rabitq4_top_neighbor`: PASS.
- `test_ec_ivf_index_placement_fewer_rerank_bytes`: PASS — **win evidence by
  counter**: table f32 = `rerank_rows * dims*4`; index f16 = `rerank_rows *
  dims*2`; index rabitq4 < table f32; equal `rerank_rows` across variants.
- `test_ec_ivf_index_placement_insert_maintains_sidecar`: PASS.
- `test_ec_ivf_index_placement_vacuum_removes_sidecar_entry`: PASS — delete +
  vacuum tombstones the dead row's sidecar entry; it is not returned and the
  survivors still rerank from the sidecar.
- `test_ec_ivf_table_placement_has_no_sidecar_and_scans`: PASS (no sidecar head;
  table-path fallback — same path a v1/v2 index takes).
- unit (page/rerank/options): `rerank_sidecar_block_roundtrips`,
  `rerank_sidecar_block_rejects_bad_payload_slab`,
  `rerank_sidecar_block_decode_rejects_wrong_tag`,
  `metadata_decode_accepts_v2_width_with_no_sidecar`,
  `f16_sidecar_payload_scores_match_table_f16_path`,
  `f16_sidecar_encoder_matches_pack_helper`,
  `coarse_rerank_accepts_index_placement_with_{f16,rabitq4}`,
  `coarse_rerank_rejects_index_placement_with_{default,explicit}_f32` — all PASS.

### Incidental fix caught by the vacuum fixture
The new `..._vacuum_removes_sidecar_entry` fixture is the first to vacuum a
`coarse_rerank` (1-bit) dense index and surfaced a pre-existing bug:
`vacuum.rs::page_payload_len` resolved the dense payload width WITHOUT the stored
`quant_bits`, defaulting to 4-bit (14 bytes) vs the 1-bit build width (13 bytes),
erroring `dense posting block payload length mismatch: got 13, expected 14`.
Fixed to pass `metadata.quant_bits` via `resolve_with_pq_group_size_and_bits`
(matching the insert-path resolution). For non-coarse formats `quant_bits` is the
build-time width, so the result is unchanged — verified by re-running the full
`vacuum` test set (rabitq / pq_fastscan / dense / diskann / hnsw / spire) green.

### Win evidence (bytes read, by EXPLAIN counter `stats_rerank_source_bytes_read`)
For dims=8, with all three variants reranking the same frontier:
- table f32: `rerank_rows * 32` bytes (dims*4).
- index f16: `rerank_rows * 16` bytes (dims*2) — exactly half.
- index rabitq4: `rerank_rows * rabitq4_payload_len` — strictly less than f32.
The pg_test asserts `index_f16 < table_f32` and `index_rabitq4 < table_f32` by
counter; exact rabitq4 payload_len is corpus-independent per dims and is asserted
as `< table f32`, not hand-quoted.
