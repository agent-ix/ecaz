# IVF Unsafe-Comment Invariant Summary

## Core Invariant Graph

- **PostgreSQL AM callback boundary.** IVF AM callbacks
  (`ec_ivf_amcostestimate`, `ec_ivf_amgettreeheight`,
  `ec_ivf_amtranslatestrategy`, `ec_ivf_amtranslatecmptype`,
  `aminsert`, `ambulkdelete`, `amvacuumcleanup`, `ambuild`,
  `ambuild_callback`, `ambeginscan`, `amrescan`, `amgettuple`,
  `amendscan`) enter through `pgrx::pgrx_extern_c_guard` wrappers.
  Raw PostgreSQL `Relation`, `IndexInfo`, `IndexScanDesc`, Datum
  array, and TID pointers are not retained beyond the guarded
  callback scope. Identical shape to SPIRE 044, HNSW, DiskANN 062.

- **Metadata at block 0, GenericXLogTxn rewriting.** IVF metadata
  lives at block 0 and is rewritten through `wal::GenericXLogTxn`
  with an exclusive metadata buffer lock. Readers borrow metadata
  page bytes only while the buffer guard pins the page. The
  `PageInit` + aligned `special_size` + `PageGetSpecialPointer`
  chain establishes the special-area read region. Established by
  packet 035; matches the HNSW invariant 2 template.

- **Page tuple line-pointer chain.** Data page tuple access follows
  the chain `page pointer → pd_lower line-pointer count → checked
  item id → tuple offset/length bounds → borrowed tuple byte
  slice`. Established by the 025–034 page series. Same shape as
  the HNSW closeout's invariant 3 — IVF's contribution was making
  the chain explicit at every reader entry (centroid tuple, list
  directory tuple, PQ codebook tuple, posting tuple).

- **Posting-list block-range traversal.** IVF-specific: posting
  lists span contiguous block ranges named by list-directory
  metadata. The range is validated against list metadata before
  the visitor helper pins each block; tuple bounds are checked
  per-tuple within the range. Visitor helpers (block sequence
  visitor, posting-ref visitor) localize the unsafe to the
  validated range entry. Established by packets 025, 027, 028.

- **Append range + exclusive-lock + WAL chain.** Posting append
  paths select a block from validated append-range logic, acquire
  an exclusive buffer lock, start `GenericXLogTxn`, register the
  buffer as a full-page image, mutate the page, finish the WAL
  transaction. Free-space measurement and
  `RecordPageWithFreeSpace` post-mutation cite the measured
  free-space basis. Established by packets 028, 029, 030, 032.

- **Scan state ownership.** The IVF scan opaque owns its stored
  query (`query_values` + `query_dimensions`), selected lists
  (`selected_list_count` + `selected_lists` array), and posting
  candidate buffer. Accessor methods (`as_query`, `as_selected_lists`)
  bind raw slice construction to the previously-stored length
  field with explicit null/empty rejection. Established by packet
  036.

- **Heap rerank scorer chain.** IVF heap rerank paths (in scan
  rerank probe, vacuum, insert) use the cross-AM scorer template:
  heap_relation + snapshot + reusable slot owned by the scorer;
  source-vector materialization copies or consumes the slot
  contents before slot reuse. Same shape as HNSW (098, 099) and
  SPIRE (082). Established by packets 038, 040, 041.

- **Reloption offset + NUL-terminated C string.** Reloption parsing
  treats `rd_options` as a PostgreSQL varlena reloptions blob;
  offsets are written by `build_local_reloptions`; string
  reloptions are stored as NUL-terminated C strings at their
  declared offsets. Identical template to SPIRE 051, DiskANN 069,
  HNSW 084. Established by packet 024.

## Lock And WAL Summary

- **Share-locked reads:** centroid, list directory, PQ codebook,
  and posting tuple reads use the read-traversal layer (packets
  025, 026, 033). Metadata reads use share-locked block-0 access.
- **Exclusive-locked mutations:** posting appends (028, 029),
  tuple rewrites (030, 032), metadata initialization and rewriting
  (035), vacuum compaction (040). Each acquires the exclusive
  buffer lock before mutation and registers the buffer in the
  generic WAL transaction before page bytes are touched.
- **WAL boundary:** `GenericXLogTxn::start` precedes
  `register_buffer`; page mutation occurs only on the registered
  full-page image; `finish` is called after all mutations are
  complete. The transaction lifecycle is stack-owned for each
  mutation scope.
- **Free-space accounting:** posting append paths call
  `RecordPageWithFreeSpace(index_relation, block_number,
  measured_free_space)` after mutation, with the measured-free-
  space basis cited at the call site.

## RAII And Resource Guards

- **`LockedBufferGuard`** (shared with HNSW): pin + lock lifetimes
  for metadata and data page access.
- **`GenericXLogTxn`** (shared with HNSW): WAL page registrations
  and finish boundary.
- **`IndexRelationGuard`**: relcache lifetime for diagnostic and
  planner snapshot reads (packet 024 admin/cost; 036 scan
  callback state).
- **Tuple slot guards**: callers own reusable heap slots used by
  heap rerank (packets 038, 040, 041).
- **PG18 read-stream state**: stack-owned for posting prefetch in
  scan paths; closed with `read_stream_end` after all buffers are
  consumed.

## Posting List And Centroid Specifics

- **Centroid scoring.** Centroid tuple reads validate the centroid
  count against metadata before slice construction. Centroid
  scoring kernels (TurboQuant 4-bit LUT, no-QJL, etc.) inherit
  their target-feature contract from the SIMD dispatch in
  `src/quant/*` (closed in packets 022, 023).
- **PqFastScan LUT dispatch.** Storage-format dispatch (Auto,
  TurboQuant, PqFastScan, RaBitQ) selects the scoring kernel; the
  unsafe inside each kernel is documented by the quant SIMD
  closeout, not by IVF.
- **Rerank mode dispatch.** RerankMode (Auto, Off, HeapF32,
  SourceColumn) determines whether heap rerank is invoked. Heap
  rerank state is acquired and released within scan rerank probe
  scope (packet 038).

## Deferred Task 50 Candidates

Convergent with the candidates listed in 083, 104, and 107:

- **AM callback guard helper.** Cross-AM `pg_am_callback!` macro
  consolidating the `pgrx_extern_c_guard` + null-check pattern.
  Prototyped by the test-only macros in packets 108–118.
- **Page tuple visitor wrapper.** A typed
  `read_page_tuple_as::<IvfCentroidTuple>(...)` or
  `with_buffer_visitor(buffer, |view| ...)` helper that absorbs
  the line-pointer chain into a safe API. Major reduction
  opportunity across IVF, HNSW, SPIRE page surfaces.
- **Posting-list block-range visitor.** IVF-specific safe iterator
  over a validated block range that yields safe tuple views,
  consolidating the visitor sites in 025/027/028 behind one
  surface.
- **Exclusive buffer + WAL transaction pair.** A
  `with_wal_full_page!(index_relation, block_number, |page| {...})`
  closure-style helper that ties the lock and WAL lifetimes
  together structurally. Shared with the HNSW append/mutate path.
- **Heap source scorer helper.** Owned safe object holding heap
  relation + snapshot + reusable slot, shared by IVF insert,
  vacuum, scan rerank — and by HNSW insert/vacuum and SPIRE
  rerank. Same candidate listed in 104.
- **Reloption offset-encoded layout type.** Typed
  `IvfReloptions` round-trip helper that absorbs the offset/
  NUL-terminated C-string reads behind a safe parse/format
  interface. Cross-AM candidate (IVF, SPIRE, DiskANN, HNSW all
  share the pattern).

## Residual Scope

No `src/am/ec_ivf` production-source entries remain. The IVF
test-only file `src/tests/ec_ivf.rs` was cleared in packet 108
via the `ec_ivf_debug!` macro consolidation, which is itself a
prototype of the AM callback guard helper Task 50 candidate.
