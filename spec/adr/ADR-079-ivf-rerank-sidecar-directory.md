# ADR-079: IVF index-side rerank sidecar needs a directory for bounded reads

Status: Proposed (2026-06-19)
Context tasks: 111g (index-side rerank placement), benchmark packet
`benchmarks/ivf-111g-115-attribution/` Finding 3.

## Context

Task 111g added `rerank_placement='index'`: a persisted compact `0x2A` sidecar
(f16 / rabitq4 payloads keyed by heap TID), intended to let the rerank stage read
a *compact* payload from the index instead of the full f32 heap source — a
claimed IO/latency win, "proven" only by the `stats_rerank_source_bytes_read`
counter.

Benchmarking (this packet, Finding 3) shows the opposite. At 100k, index-side f16
vs table-side f16:

| metric | table-side | index-side |
|---|---|---|
| p50 @ nprobe 8 | 4.7 ms | **551 ms** (117×) |
| p50 @ nprobe 200 | 13.2 ms | **552 ms** (42×) |
| latency vs nprobe | scales 5→13 ms | **flat ~540 ms** |
| index size | 25.4 MiB | **416 MiB** |

**Root cause** (`scan.rs::rerank_probe_candidates_index_side` →
`load_rerank_sidecar_payloads`): every query walks the **entire** `0x2A` chain via
`next_tid` and decodes/copies **every** payload into a
`HashMap<ItemPointer, Vec<u8>>`, then looks up only the ~`rerank_width` (64)
survivors. For N=100k f16 (3072 B each) that is ~307 MB read + 100k allocations
per query to serve ~196 KB — O(N), nprobe-independent (hence the flat ~540 ms).
The counter reported only the O(W) survivor bytes, so it never saw the real cost.

The build chain is globally TID-sorted into blocks of `entries_per_block`
(`build.rs::build_rerank_sidecar_chain`); inserts prepend a single-entry block at
the head (`insert.rs::append_rerank_sidecar_entry`). The chain is a singly linked
list — **no random access** — so any read is O(chain).

## Decision

Add a **sidecar directory** that maps heap-TID ranges to sidecar block pointers,
enabling **survivor-directed bounded reads**: read only the blocks that actually
contain the (≤ `rerank_width`) survivors, not the whole chain.

### On-disk (research project — clean format break, just rebuild)

- New metadata field `rerank_sidecar_directory_head: ItemPointer`. **Correction
  (per codex 111g/004 P1):** v3 metadata is **full** at `EC_IVF_METADATA_BYTES ==
  86` with `rerank_sidecar_head` already occupying bytes `80..86` — there is **no
  spare ItemPointer tail**. Adding the directory head therefore requires a
  metadata **size/layout change**: extend `EC_IVF_METADATA_BYTES` to 92 (new field
  at `86..92`), add `EC_IVF_METADATA_RERANK_SIDECAR_DIRECTORY_HEAD_OFFSET = 86`,
  bump the encode/decode + the `size_of_assertions` / on-disk fixtures, and (per
  research-project posture) treat it as a **clean format break + rebuild** (no
  back-compat path; the decode requires the new width). Do NOT "add at the next
  free offset, bump nothing" — that would over-read/over-write the 86-byte area.
- A directory chain of blocks, each holding a sorted run of
  `(first_heap_tid, block_tid)` entries — one entry per build-written sidecar
  block, ascending by `first_heap_tid` (build already writes blocks in TID
  order). Small: `ceil(N / entries_per_block)` entries total.
- Inserts: keep prepended single-entry sidecar blocks as a bounded **unsorted
  insert prefix** tracked by the existing `rerank_sidecar_head` up to the first
  build block; the directory covers the (sorted, immutable) build blocks. REINDEX
  folds inserts back into the sorted build chain + directory.

### Build / Insert / Scan

- **Build:** while emitting sidecar blocks, record `(first_heap_tid, block_tid)`;
  after the chain is written, emit the directory chain and set
  `rerank_sidecar_directory_head`.
- **Insert:** unchanged (prepend), but the prepended region stays small between
  REINDEXes; scan handles it linearly.
- **Scan (`rerank_probe_candidates_index_side`):** survivors are already
  TID-sorted. (1) Linearly scan the insert prefix (head → first build block),
  collecting any survivor payloads. (2) Load the directory (small), and for the
  remaining survivors binary-search to the owning build block and read **only**
  those blocks, extracting the survivor payloads. Bound: O(W) block reads +
  O(insert-prefix). Replace the full-chain HashMap entirely.

### Expected result

Index-side reads drop from ~307 MB to ~(W × payload_len) ≈ 196 KB per query;
latency should fall from ~540 ms to ≤ table-side (compact f16 read < heap f32
read), realizing 111g's original thesis. Re-bench `sidecar-index-placement` to
confirm before promoting `rerank_placement='index'`.

## Consequences

- Index-side becomes a real option (compact payload + bounded read); table-side
  remains the lean-index default.
- Adds a directory structure + a small REINDEX-to-compact story for inserts.
- The `stats_rerank_source_bytes_read` counter must additionally record blocks/
  pages actually read (not just survivor payload bytes) so the evidence reflects
  true IO — the original counter bug that hid this regression.

## Alternatives considered

- **Per-relation cache of the full payload map:** amortizes O(N) across warm
  queries but costs ~307 MB/backend and still O(N) cold; a workaround, not a fix.
- **Co-locate the compact rerank payload inside the dense posting block** (read
  during the coarse scan, zero extra fetch): best long-term, but a larger
  posting-format redesign — deferred.
