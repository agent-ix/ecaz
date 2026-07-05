## Feedback: PqFastScan Live Insert On Built Indexes

Read `derive_pq_fastscan_search_code_for_insert` at `insert.rs:1493`,
`append_pq_fastscan_tuple` at `:1543`, the fallback-page path at
`:1743`, `coalesce_duplicate_grouped_heap_tid` at `:2047`, and the
`run_insert_with_adapter` empty-index branch at `:456-530`.

### What's right

- **Correctly splits the unsupported boundary into two pieces.**
  Empty-index grouped insert remains rejected (no persisted
  codebooks), but built-index grouped insert now has a real success
  path. That's the honest architectural cut — the previous blanket
  reject was over-rejecting.
- **Grouped append reuses
  `build::stage_v2_grouped_build_payload(...)` for payload
  shaping.** This keeps build-time and insert-time payload layouts
  in lockstep by construction. The task-15 reviewer feedback list
  from 310–333 called out "collapse duplicate grouped-code packing"
  as a shared-contract concern; reusing the build helper here is
  the right answer to that concern.
- **Duplicate coalescing mutates only the inline heap-TID list.**
  Rerank and neighbor payloads stay untouched. That's correct — a
  duplicate by definition has the same code and the same neighbor
  set, only the heap TID differs.
- **Derived layout is validated against metadata.** The search-code
  length check at `:1514-1520` and the binary-sidecar length check
  at `:1567-1573` would catch a codebook/metadata mismatch loudly
  instead of silently writing wrong-shape tuples. Good
  defense-in-depth.

### Concerns

1. **Hot/rerank/neighbor must fit on one fresh page.** The explicit
   error at `:1600-1604` — "tqhnsw aminsert does not yet support
   PqFastScan tuples that require more than one fresh data page" —
   is honest but a real functional limit. For large
   `binary_word_count + search_code_len + neighbor slot count` at
   high M, this boundary may bite at corpus scale. Needs a
   measurement pass on realistic M/dimension combinations before
   task 15 merges.

2. **No lock-ordering comment on the grouped append path.** Task 15
   calls out "Follow ADR-026 lock ordering (layer-0 backlink lock
   before upper-layer write locks)." The adapter threads the same
   underlying scaffolding as scalar insert so the ordering should
   be inherited, but there's no inline comment claiming that
   invariant. If ADR-026 is load-bearing for correctness, the
   grouped append should have at least one asserting comment or a
   proof-test that exercises the backlink-first ordering under
   concurrent insert.

3. **Duplicate scan under SHARE locks; race window preserved.** The
   comment at `insert.rs:550-553` documents the "concurrent insert
   may double-insert the same code" race as an accepted trade for
   removing the metadata serialization point. That applies to both
   formats. Worth flagging that the grouped duplicate coalescing
   path inherits this race — the impact is cosmetic (two hot tuples
   for the same code) but it means vacuum must be able to handle
   that shape, which isn't explicitly tested.

4. **Linker gap.** Live insert is the first functional parity
   checkpoint for grouped storage, and its correctness cannot be
   verified via clippy + `cargo check --tests`. Running the new pg
   tests (`insert appends`, `duplicate coalesces`, `empty-index
   rejects with codebook-specific error`) somewhere — any CI lane
   that has linkable PG symbols — is the load-bearing validation
   for this packet.

### Observation

This is the first packet where grouped storage actually becomes a
peer of scalar for a real lifecycle operation. The quality bar for
the next few packets (383 vacuum, 390 bootstrap) rides on the
ordering and race assumptions in this one being right.
