# Task 167 M5 — on-disk incremental-insert orchestration design

The pure planning pipeline is complete and unit-tested (packet 001, 8 tests):
`rank_insert_candidates` → `select_insert_forward_neighbors` →
`plan_insert_backlink` → `build_insert_node_tuple`. This note specifies the
on-disk orchestration that glues them to the relation, with every dependency
verified against the current code, so the mutation slice can be implemented
directly. **No heap reads are needed** — candidates come from the FR-080
head-sample region (full-precision vectors already persisted).

## `graph_insert_record(index, new_vec_id, new_heap_tid, new_source_vector)`

1. **Metadata + codec.** `read_metadata_from_index_handle`; `metadata_code_len`.
   Reconstruct the codec binding from `metadata.neighbor_codec_kind` +
   `metadata.seed` + `dimensions`. RaBitQ (the default) and TurboQuant are
   *seeded* — `DistannCodecBinding::prepare(fmt, &[], dims, seed)` with no source
   refs. GroupedPq is *trained*: rehydrate from the persisted codebook
   (`read_grouped_codebooks_from_relation`) — a follow-up; error clearly until
   then. `new_code = binding.encode(new_source_vector)`.
2. **Candidate search.** `read_head_samples_from_relation(head_sample_head,
   dims, head_index_cap)` → `rank_insert_candidates(new_source_vector, samples,
   build_list_size_l)`. For a node count within `head_index_cap` the samples are
   every node, so the edges are exact; larger indexes seed from the head region
   (greedy-walk scaling is a follow-up). Drop a sample whose vec_id == the new
   one (re-insert).
3. **Forward edges.** `select_insert_forward_neighbors(new_source_vector,
   candidates, alpha, graph_degree_r)` → forward vec_ids.
4. **Forward neighbor codes.** `read_directory_from_relation` →
   `directory_lookup(vec_id)` → `read_raw_tuple_bytes_from_relation` →
   `DistannNodeTuple::decode` → each neighbor's `search_code`. Feed
   `build_insert_node_tuple` → the new node record.
5. **Append node.** WAL-logged fresh-page append (the `dml::append_delta_tuple`
   pattern, node payload) → `new_tid`.
6. **Backlinks (append-if-free).** For each forward neighbor: `visit_tuple_bytes_mut`
   (the `dml::set_tombstone_flag` pattern) to append `(new_vec_id, new_code)`
   into a free adjacency slot and bump `neighbor_count`; skip when the neighbor
   is at `graph_degree_r` (the full-reprune path — `plan_insert_backlink`'s
   robust_prune branch — needs neighbor source vectors, so it lands with the
   head-sample-backed reprune follow-up). Same-length in-place write; records are
   fixed size.
7. **Directory.** Insert `(new_vec_id, new_tid)` keeping the sorted order.
8. **Metadata.** `node_count += 1`; `directory_head` if it moved;
   `overwrite_metadata_page_handle`.

## The one gap: directory maintenance primitive

`read_directory_from_relation` yields a sorted `Vec<(vec_id, tid)>` and
`directory_lookup` binary-searches it — so a new entry must land in sorted
order. `stage_directory_chain` + `write_data_pages` build a directory chain, but
`DataPageChain` numbers pages from `FIRST_DATA_BLOCK_NUMBER` (block 1) while
`write_data_pages` appends via `P_NEW` at the relation's current end — so the
chain's internal `next_tid`s only line up for a *fresh* index, not an
incremental append. Two clean options, either landed as its own committed
storage slice first:

- **(A) Base-block chain write.** Add `DataPageChain::with_base_block(base,
  page_size)` (or a `write_data_pages_at(handle, chain, base)` that rewrites
  tids by `base - FIRST_DATA_BLOCK_NUMBER`). Then rebuild the sorted directory
  into a chain based at `main_fork_block_count_handle(handle)`, `write_data_pages`,
  and repoint `directory_head`. Old directory pages become dead space (fine;
  research, no compaction). O(n) per insert — acceptable for the maintenance-
  triggered fold; a delta drain amortizes it over a batch.
- **(B) Insert-tail directory.** A small unsorted tail chain (new metadata head)
  that `directory_lookup` linear-scans after the binary search; drained into the
  sorted directory at the next epoch build. O(1) append, O(tail) lookup.

(A) is simpler and matches the "rebuild for correctness first, incremental tail
later" plan in packet 001; (B) is the scaling form.

## Wire

A maintenance `#[pg_extern] ec_distann_fold_delta_into_graph(index)` graph-
inserts each delta-buffer entry (D5) via `graph_insert_record`, then clears the
folded entries — bounding the buffer and giving inserted rows graph
connectivity. Multi-node: the coordinator routes each fold to the hash-owning
node through the FR-083 write endpoint (the `apply_record_writes` surface,
extended with the new-record op). `aminsert` may trigger a fold when the buffer
crosses a threshold, or it stays operator-triggered.

## Test plan (committed-DB / pg_test)

Build a fixture, insert a vector, `ec_distann_fold_delta_into_graph`, assert:
`node_count` +1; directory resolves the new vec_id; the new record decodes with
the expected forward edges; at least one forward neighbor gained the backlink;
an empty-delta-buffer scan reaches the new row via graph traversal (not the
exact-scan tail). Then A/B the folded index's recall vs a full rebuild on the
same rows (should match within noise).
