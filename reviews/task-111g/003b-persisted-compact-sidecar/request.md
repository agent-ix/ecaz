# Task 111g Packet 003b — Persisted compact rerank sidecar (`rerank_placement='index'`)

**Role:** coder
**Branch:** `task-111g-coarse-rerank-representations`
**Base SHA:** `0710f9f4c` (001 dispatch + 003a design + reviewer blessing)
**Code SHAs:** `3f9e990c0` (sidecar build/insert/scan + metadata v3 + docs),
`1ca3ae0f3` (vacuum sidecar maintenance + coarse_rerank vacuum payload_len fix)
**Status:** implementation complete; requesting review. Coder does NOT merge.

Implements the design blessed in
`reviews/task-111g/003a-persisted-compact-sidecar-design/feedback/2026-06-18-01-reviewer.md`:
Option B (compact sidecar keyed by heap TID, tag `0x2A`), the B1 metadata
version bump v2→v3, byte-reduction-by-counter win evidence, and the filtered
tid-sorted survivor read. Index placement IS the win (reviewer scope
correction): `rerank_placement='index'` is now the persisted `0x2A` sidecar;
`'table'` stays the heap-source path.

## Sidecar storage shape

New tuple tag `0x2A` (`IVF_RERANK_SIDECAR_BLOCK_TAG`), an index-internal block
in its own page chain, keyed by heap TID:

```
[tag:u8=0x2A][rerank_format:u8][payload_len:u16][entry_count:u16][next_tid:6]
[heap_tids: entry_count * 6]            (ascending within a build-written block)
[payloads:  entry_count * payload_len]
```

- **f16** payload_len = `dimensions * 2` (the exact IEEE binary16 round-trip of
  the source, packed LE) — half the `dimensions * 4` f32 heap source.
- **rabitq4** payload_len = the rabitq4 codec payload length (the same compact
  code the on-the-fly path produces, now persisted; scored directly, no
  re-encode).
- **f32** = no sidecar (keeps the heap source; `heap_f32`/`table` stay
  bit-identical).

Blocks chain via a per-block `next_tid` (like the PQ-codebook chain). The chain
head is the new metadata field `rerank_sidecar_head`.

## Lifecycle

- **Build** (`build.rs::build_rerank_sidecar_chain`): after postings/directory,
  encodes each row's compact payload, sorts globally by heap TID, packs into
  `0x2A` blocks, links them, and records the head in metadata. Only when
  `rerank_placement='index'` and a compact `rerank_format`.
- **Insert** (`insert.rs::append_rerank_sidecar_entry`): prepends a single-entry
  `0x2A` block (`next_tid = old head`) and repoints `metadata.rerank_sidecar_head`
  — O(1), no tail walk. Reads the declared reloptions for placement/format
  (metadata does not persist those); the empty-bootstrap build path now also
  reads reloptions so the first-row build's sidecar matches the index
  definition.
- **Vacuum** (`vacuum.rs::bulkdelete_rerank_sidecar`): walks the chain and
  tombstones dead heap TIDs in place (`heap_tid = INVALID`, same byte length, so
  the rewrite fits the existing slot). Space is reclaimed on REINDEX, which
  regenerates the chain sorted. Correctness does not depend on this — dead heap
  TIDs never appear as rerank survivors (they come from live postings vacuum
  already prunes) — the tombstone defends against heap line-pointer reuse.
- **Scan** (`scan.rs::rerank_probe_candidates_index_side`): when placement is
  index and a sidecar head exists, loads the chain into
  `HashMap<heap_tid, payload>`, sorts survivors tid-ascending, and reads only the
  survivor payloads (bounded by `rerank_width`). Reuses `RerankScorer`
  (`score_sidecar_payload` for f16, `score_sidecar_payloads_batch` for rabitq4) —
  no new SIMD. Records `stats_rerank_source_bytes_read`.

## Metadata v3 change + backward compat

- `EC_IVF_INDEX_FORMAT_VERSION` 2 → 3; `EC_IVF_METADATA_BYTES` 80 → 86; new
  `EC_IVF_METADATA_RERANK_SIDECAR_HEAD_OFFSET = 80`; `EC_IVF_METADATA_BYTES_V2 = 80`.
- `decode` accepts buffers `>= 80` and version `1..=3`; a v1/v2 image (80-byte
  metadata special area) decodes with `rerank_sidecar_head = INVALID`. The
  metadata read/update clamp the slice to the **actual** special-area size
  (`pd_special`-derived), so a v2 page (80-byte special area) is neither
  over-read nor over-written.
- An existing v2 index scans with no sidecar: the scan sees `head == INVALID`
  and falls back to the table/heap_f32 path (bit-identical f32 results).

Backward-compat is proven three ways:
- unit: `page.rs::metadata_decode_accepts_v2_width_with_no_sidecar`;
- golden fixture: `tests/on_disk_fixtures.rs::ivf_metadata_v1_fixture_decodes`
  now asserts the committed **80-byte v1 on-disk image** decodes under the v3
  reader with `rerank_sidecar_head == INVALID`;
- pg_test: `test_ec_ivf_table_placement_has_no_sidecar_and_scans` (a
  table-placement index carries no sidecar head and still scans, the same
  fallback an old index takes).

## Hard constraints honored

- `heap_f32` / `rerank_placement='table'` results stay bit-identical (no sidecar
  on that path; the f32 scorer is unchanged).
- Coarse stage, dense block layout (`0x25`/`0x28`), coalescing, quant math:
  untouched. The only scan change is which bytes the rerank loop reads.
- No stripped dead formats reintroduced (`0x26`/`0x27`/`0x29`/page-scatter).
- No new SIMD — scoring routes through the existing `RerankScorer` +
  `candidate_batch` scorers.

## Validation (PG18; no bench env)

See `artifacts/manifest.md` for commands + key lines.

- `cargo check --no-default-features --features pg18` — clean.
- `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
  — clean.
- Integration tests `size_of_assertions` / `on_disk_fixtures` / `upgrade_matrix`
  — pass (v3 offsets pinned, v1 fixture backward-read, matrix reconciled).
- pg_test fixtures (PG18), all asserting via the EXPLAIN counter / debug
  surface, NO `ecaz bench suite`, NO fabricated numbers (NFR-007):
  - `test_ec_ivf_index_placement_f16_rabitq4_admin_snapshot` — index placement
    creatable + reported.
  - `test_ec_ivf_index_placement_f16_matches_table_f16_ranking` — recall parity:
    index-side f16 top-5 ranking AND bit-identical scores vs table-side f16.
  - `test_ec_ivf_index_placement_rabitq4_top_neighbor` — rabitq4 index
    correctness.
  - `test_ec_ivf_index_placement_fewer_rerank_bytes` — **the win evidence**:
    index f16 reads `dims*2`/candidate and index rabitq4 reads the rabitq4
    payload length, both strictly fewer than table f32's `dims*4`/candidate, on
    the same corpus/query with equal `rerank_rows` (by counter).
  - `test_ec_ivf_index_placement_insert_maintains_sidecar` — a post-build insert
    is reranked from the sidecar and ranks first.
  - `test_ec_ivf_index_placement_vacuum_removes_sidecar_entry` — delete + vacuum
    tombstones the dead row's sidecar entry; survivors still rerank.
  - `test_ec_ivf_table_placement_has_no_sidecar_and_scans` — backward-compat
    fallback.
  - unit: sidecar block round-trip/reject, v2-width metadata decode, f16
    sidecar-score == table-f16, index-placement accept/reject in options.

## Incidental fix (caught by the vacuum fixture)

The vacuum fixture is the first to vacuum a `coarse_rerank` (1-bit) dense index
and surfaced a pre-existing bug: `vacuum.rs::page_payload_len` resolved the dense
payload width without the stored `quant_bits` (defaulting to 4-bit), erroring
`dense posting block payload length mismatch: got 13, expected 14`. Fixed to pass
`metadata.quant_bits` (matching the insert path). Non-coarse formats are
unaffected (verified by the full `vacuum` test set staying green).

## Open decisions to check

1. **Vacuum tombstone vs compaction.** Dead sidecar entries are tombstoned in
   place (same byte length) rather than removed, because entry removal changes
   the tuple length (no safe same-page rewrite) and correctness does not require
   it. Space is reclaimed on REINDEX. Flagging in case you want eager
   compaction instead.
2. **Insert prepends (chain not globally sorted after inserts).** The scan reads
   via a HashMap lookup, so order does not affect correctness; build writes
   sorted, inserts prepend. The 003a design left the merge-join-vs-lookup choice
   open; I chose the lookup (simplest, O(1) insert, survivor-bounded read).
3. **Upgrade matrix fixture reuse.** The new ivf v2/v3 matrix rows point at the
   existing `ivf_metadata_v1.hex` fixture (mirroring the hnsw precedent where v1
   and v3 rows share one metadata fixture). No new hand-authored hex fixture was
   fabricated; the v1 fixture is the durable backward-read proof.

## Update — backward-compat machinery stripped (2026-06-19)

This is a research project with no users and no backward-compat requirement, so
the metadata bump is now a **clean break to format v3** (old indexes are simply
rebuilt). The 003b backward-readable machinery was removed; the `0x2A` rerank
sidecar, v3 86-byte metadata, and the legitimate `rerank_sidecar_head = INVALID`
"no sidecar -> table/heap source" runtime state are all KEPT.

Removed:

- `src/am/ec_ivf/page.rs`: `EC_IVF_METADATA_BYTES_V2`,
  `EC_IVF_INDEX_FORMAT_VERSION_MIN`, `special_size()`, `metadata_special_bytes()`
  and its clamp; decode now requires `len >= EC_IVF_METADATA_BYTES (86)` and
  `format_version == 3` (rejects everything else); the v2-image decode unit
  tests; the v1/v2/additive comment framing.
- Re-exports of `EC_IVF_METADATA_BYTES_V2` from `src/am/mod.rs`, `src/lib.rs`,
  `src/am/ec_ivf/mod.rs`.
- `tests/size_of_assertions.rs`: the `EC_IVF_METADATA_BYTES_V2 == 80` assertion.
- `tests/on_disk_fixtures.rs`: the v1-fixture decode test; the byteswapped-version
  test and the decode test now use a clean v3 fixture
  (`fixtures/on-disk/ivf_metadata_v3.hex`); `fixtures/on-disk/ivf_metadata_v1.hex`
  deleted.
- `fixtures/upgrade/matrix.csv`: ivf now lists only `3` (read+write).
- `docs/on-disk-format.md`, `plan/tasks/42-on-disk-format-invariants.md`:
  simplified to v3-only, dropping the 1/2/3 history and "backward-readable"
  policy framing.

Validation (logs under `artifacts/`): `cargo check` and
`cargo clippy --all-targets -D warnings` clean (pg18); integration tests
`size_of_assertions` (13), `on_disk_fixtures` (47), `upgrade_matrix` (2) all
pass; `cargo pgrx test pg18` coarse_rerank set (18) and index/table placement +
sidecar set (7) all pass. One unrelated SPIRE test
(`pg_test_ec_spire_boundary_replica_placement_diagnostics_sql`) matched the
`placement` name filter and fails on a pre-existing ec_spire strict-snapshot
assertion — no ec_spire files were touched by this change.
