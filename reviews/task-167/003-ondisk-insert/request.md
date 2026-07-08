# Review request — Task 167 M5: on-disk incremental graph insert

**Branch:** `task-165-ec-distann-m3`. The M5 core: folding delta-buffer inserts
into the persisted Vamana graph, end-to-end and tested.

## What landed

`graph_insert_record` (insert.rs) — the on-disk orchestration that applies the
tested pure pipeline (rank → select → assemble) then mutates the index:

1. Reconstruct the codec from `metadata.neighbor_codec_kind` + `seed` (seeded
   RaBitQ/TurboQuant; GroupedPq rehydration errors clearly as a follow-up) and
   encode the new node's search code.
2. Candidate search over the FR-080 head-sample region (**no heap reads**) →
   forward-neighbor selection.
3. Read each forward neighbor's embedded search code; assemble the new node
   record; **append it** WAL-logged on a fresh page.
4. **Append-if-free backlinks** on each forward neighbor (in-place WAL write;
   a full neighbor is left unchanged — the forward edge preserves connectivity;
   full-reprune is a head-sample-backed follow-up).
5. **Rebuild the sorted directory** into a base-block `DataPageChain` at the
   relation's current end (the primitive committed earlier) + `write_data_pages`;
   repoint `directory_head`; `node_count += 1`.

Wired via `ec_distann_fold_delta_into_graph(index)` — graph-inserts every delta
entry (D5) then drains the buffer.

## Evidence (`artifacts/test-evidence.log`)

`test_ec_distann_fold_delta_into_graph`: insert a row (delta buffer) → fold →
`node_count` +1, delta buffer drained, and the row is found via **graph
traversal** (empty delta buffer) — proving true graph connectivity, not the
exact-scan tail. The scan reads the rebuilt directory + node record by real
on-disk TIDs. **101 distann pg_tests pass, 0 failed; clippy clean.**

## Follow-ups (noted, not blockers)

- GroupedPq codec rehydration for insert (codebook read).
- Full-reprune backlinks when a neighbor is at capacity (needs neighbor source
  vectors — head-sample-backed).
- Multi-node fold routing via the FR-083 write endpoint; head-sample refresh so
  inserted nodes seed later inserts; greedy-walk candidate search past the
  head-index cap.
- `materialize_chain_from_index_handle` re-packs tuples (build-contiguous
  assumption) so it can't read a post-insert layout; the relation-TID readers
  (scan path) are unaffected. A materialization that honors real block numbers
  is a test-infra follow-up.

## Ask

Review the on-disk mutation ordering (append node → backlinks → directory →
metadata), the WAL writes, and the follow-up boundary. Not closing.
