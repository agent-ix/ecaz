# Review request — Task 162 M0: ec_distann scaffold, build, and local scan

- Branch: `task-162-ec-distann-m0`, head `6e8e58572`
- Commits: `29e88ebd4` → `6e8e58572` (5 code commits; see
  `artifacts/manifest.md`)
- Scope: the code half of milestone M0 (single-node parity). Bench
  evidence (parity A/B, NFR-018 storage, C sensitivity, G0 kill-check)
  follows in packet 002+ — this packet is the static/code review of the
  new AM.

## What landed

1. **FR-075 AM surface** (`src/am/ec_distann/{mod,routine,options,cost}.rs`,
   `sql/bootstrap.sql`): fifth access method with the full callback set,
   reloptions (`graph_degree`, `build_list_size`, `alpha`,
   `neighbor_code_format` [D7 GroupedPq default], `closure_epsilon`,
   `head_index_cap` [D3 default 4096], `source_identity`), GUCs
   `ec_distann.{beam_width,hop_rounds,top_k,scan_profile_notice}`,
   ecvector+tqvector opclasses.
2. **FR-076 lean record** (`tuple.rs`): tag 0x09 — vec_id, heap_tid,
   tombstone flags, fixed-stride search code, neighbor vec_ids +
   embedded neighbor codes. Structurally NO full-precision vector (D11);
   encoded size independent of dimension (AC-6). Pooled `decode_into`.
3. **D6 identity** (`identity.rs`): vec_id = stable murmur3-fmix64
   hash64 with domain-tagged global (16-byte ADR-063 payload) and local
   (heap-TID) modes; pinned-value tests make hash changes a format
   decision. `source_identity='include'` DDL wiring is NOT in this
   packet (build errors with a clear message; ADR-063 canonicalization
   lands before M2 — the M0 bench corpora have no identity column).
4. **Monolithic build** (`ambuild.rs`): seed-deterministic single-shard
   FR-077 degenerate case over the shared
   `build_vamana_graph_with_stats`/`approximate_medoid` core; D6
   collision detection; codec encode via `DistannCodecBinding`
   (grouped_pq trains + persists codebook chain via the shared diskann
   stager; rabitq/turboquant seeded at ec_diskann-equal widths); sorted
   vec_id→TID directory chain; FR-080 entry-region BFS sample (vec_id +
   vector, capped at C); 72-byte metadata page.
5. **FR-080 head index + FR-081 local loop** (`head_cache.rs`,
   `scan.rs`, `expand.rs`, `routine.rs`): per-backend cache (directory +
   deterministic in-memory head Vamana + flat codebooks) keyed by index
   oid + metadata fingerprint; eager rescan orchestration (head descent
   seeds → ≤H rounds of best-BW expansion; vec_id dedupe; BW×H hard
   cap; sub-k-on-exhaustion complete; D9 early-exit); local expander =
   one record read + one co-placed heap read per expansion (D11);
   per-query counters behind `ec_distann.scan_profile_notice`.

## The frozen expansion seam (M0 design output — please review hard)

`scan.rs::DistannNodeExpander::expand_nodes(vec_ids, code_threshold) ->
Vec<DistannExpandedNode { vec_id, exact_dist: Option<f32>, is_tombstone,
heap_tid, neighbor_vec_ids, neighbor_code_dists }>`

This mirrors FR-079's `ec_distann_expand_nodes` wire contract
one-to-one (request-order responses covering every requested vec_id;
unresolvable vec_id = error, never a silent miss; tombstones may omit
the vector read but return edges). `heap_tid` is documented as
local-only materialization convenience, NOT part of the wire contract.
The M2 remote form is intended to implement this same trait by grouping
the batch per owning node and issuing one pooled SQL call per node —
the orchestration loop does not change. If this shape is wrong, now is
the time.

## Deliberate M0 postures (call out if you disagree)

- `aminsert` errors (FR-083 delta-buffer slice is next); bulkdelete is
  a stats no-op (D10: nothing reclaimed within a published epoch).
- Head-cache invalidation is a metadata fingerprint, valid only because
  M0 content is frozen between REINDEXes; FR-082 epochs replace it.
- FR-081-AC-5 counters surface via NOTICE GUC, not EXPLAIN — the
  CustomScan EXPLAIN surface arrives with the lifted SPIRE shell (M2+).
- Early-exit uses approximate code distances as the improvement bound
  (as spec'd); AC-4 equivalence is asserted empirically at bench time.
- Shared scan mechanics (`DiskannScanDescView`, `set_scan_heap_tid`,
  `exact_heap_rerank_distance`, `source_inner_product*`,
  `write_data_pages`, codebook stager) were re-exported pub(crate) from
  ec_diskann rather than forked; their error strings still say
  "ec_diskann" — flagged as a follow-up `am::common` lift.
- Directory resolution is a per-backend cached Vec + binary search;
  head search list size = max(2·BW, 32) — both provisional, measured in
  packet 002.

## Validation

- `cargo clippy --all-targets --no-default-features --features pg18 --
  -D warnings`: clean (`artifacts/clippy-pg18.log`).
- `cargo pgrx test pg18 ec_distann`: **42/42 green**
  (`artifacts/pg18-ec-distann-tests.log`), covering: AM registration,
  create/drop/reindex, reloption validation (FR-075-AC-2), record
  round-trip byte-exactness (FR-076-AC-1), REINDEX vec_id identity
  (AC-2), embedded neighbor code == neighbor search code and search
  code == direct codec encode (AC-3), dimension-independent record size
  (AC-5/6), neighbor_count bound (CON-2), directory
  ascending/resolving, BFS head sample determinism across REINDEX
  (FR-080-AC-2) and cap coverage, ordered-scan score monotonicity
  (FR-075-AC-3), planner-driven self-recall for rabitq and grouped_pq,
  loop invariants via mock expander (no double expansion, BW×H cap,
  tombstone edges traversable / excluded from results, early-exit,
  sub-k exhaustion), GUC defaults, DML posture errors.
- Not run: full non-distann pg18 suite. The only shared-code edits are
  visibility bumps (pub(super)→pub(crate)) with no behavior change;
  static review deemed sufficient per the checkpoint policy. Known
  pre-existing failures on this host (turboquant counters test, GUC
  test flakes under parallel threads) are documented in the Task 168
  closeout and untouched here.

## Asks

1. Seam shape (above) vs FR-079 — any mismatch that would force a remote
   rework?
2. FR-076 record layout / 72-byte metadata page — field or offset
   objections before bench data makes rebuilds annoying?
3. The pub(crate) reuse-vs-lift call on ec_diskann scan mechanics.
4. Anything in the M0 postures list that should block the bench packet.
