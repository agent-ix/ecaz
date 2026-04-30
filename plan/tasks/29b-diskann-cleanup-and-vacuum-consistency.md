# Task 29b: DiskANN Cleanup and Vacuum Consistency

Status: planned, follows Task 29a (binary-sidecar prefilter)
Owner: coder1 / runtime-index track
Backstory: `review/11099-task29-diskann-landing-readiness/feedback.md`

## Goal

Close the loose ends from the Task 29a binary-sidecar swap so the
DiskANN scan and maintenance paths agree on which prefilter scores
candidates, and tighten the small code-quality items that came out
of the merge-readiness review.

This is **not** a removal task. Grouped-PQ stays — it's load-bearing
for `ec_hnsw` and `ec_ivf`, and the DiskANN fallback path is the
GUC-controlled emergency rollback for the binary sidecar. What we
fix here is consistency (vacuum still uses grouped-PQ even when the
index has a sidecar), polish (code shape, tests), and verification
(SIMD codegen).

## Background

Task 29a swapped the **scan-path** prefilter from grouped-PQ to the
persisted binary sidecar (`tuple.binary_words`) for every DiskANN
index built with `has_binary_sidecar = true`. Recall went from
~0.93 to ~0.997 at default reloptions.

But the swap only touched `routine.rs amrescan`. Vacuum repair's
candidate planner (`plan_vacuum_fill_candidates_for_target` at
`routine.rs:1305-1411`) **still scores via grouped-PQ** through
`build_grouped_pq_lut_from_persisted` and `grouped_pq_score_f32`.

This produces a quality split: a freshly-vacuumed neighbor slot may
hold candidates that grouped-PQ ranked highly but binary-sidecar
would have ranked lower. The graph degrades subtly in the same
failure mode that motivated the entire 29a series — except now only
during vacuum, and only on the slots being repaired.

The grouped-PQ scan path is also still wired in via the `Auto`
fallback and the explicit `GroupedPq` GUC choice. Per the directive
in `review/11099`'s landing-readiness feedback, that's the right
production posture (emergency rollback knob), so the path stays —
but the GUC documentation and pgrx test coverage need to catch up
to that decision.

## Scope

### 1. Vacuum-repair sidecar wiring

`plan_vacuum_fill_candidates_for_target` builds a target-side prefilter
score by SRHT-rotating the target's heap source vector and computing
a grouped-PQ LUT. The mirror change for the sidecar path is
straightforward:

- When `metadata.payload_flags & PAYLOAD_FLAG_BINARY_SIDECAR != 0`,
  derive the target's binary words from `SRHT(target_source_vector)`
  via `pack_query_sign_bits` (same helper the scan path uses), and
  pass `hamming_xor_popcount(target_binary_words, &tuple.binary_words)`
  as the `prefilter` closure to `greedy_descent_with`.
- Otherwise, keep the existing grouped-PQ codepath unchanged.

The `greedy_descent_with` shell does not change — the prefilter
closure parameter is the only coupling point.

The same `ec_diskann.prefilter_kind` GUC should also gate vacuum
repair: `Auto` honors the flag, `BinarySidecar` errors when the flag
is missing, `GroupedPq` forces the legacy path. Use the same
`use_binary_sidecar` decision helper as the scan path so the two
sites cannot drift.

**Files**:
- `src/am/ec_diskann/routine.rs` — split `plan_vacuum_fill_candidates_for_target`
  into a prefilter-selection step + a `greedy_descent_with` step
  driven by either prefilter, mirroring the `amrescan` pattern.

**Validation**:
- `cargo test --lib am::ec_diskann::vacuum`
- `cargo pgrx test pg18 test_ec_diskann_vacuum_refills_broken_neighbor_slot`
- A new fixture-driven measurement: run an isolated real-10k vacuum
  scenario (kill ~5% of nodes, run vacuum, measure recall@10 against
  the truth cache before/after vacuum). Goal is to confirm vacuum
  repair under the sidecar prefilter does not regress recall vs the
  pre-vacuum state. If grouped-PQ vacuum was also recall-neutral on
  this corpus we won't see a delta — that's fine, the goal is
  consistency not necessarily a measured win.

### 2. SIMD verification on `hamming_xor_popcount`

`scan_query.rs:128-134` is the per-visit hot path:

```rust
pub fn hamming_xor_popcount(query_words: &[u64], candidate_words: &[u64]) -> u32 {
    query_words
        .iter()
        .zip(candidate_words.iter())
        .map(|(query, candidate)| (query ^ candidate).count_ones())
        .sum()
}
```

At fixed 24-u64 length (1536-d corpora), the compiler should
auto-vectorize and emit `popcntq` inside a tight unrolled loop.
Compare to pgvectorscale's `distance_xor_optimized`
(`pgvectorscale/src/access_method/distance/mod.rs:255-323`), which
uses an explicit match-arm-per-length unroll specifically to give
the compiler exact size hints.

**Steps**:
1. `cargo asm --no-default-features --features pg18 --release ecaz::am::ec_diskann::scan_query::hamming_xor_popcount`
   on x86_64 with `-C target-cpu=native` to inspect generated code
   for a 24-u64 input.
2. If `popcntq` lands inside a tight loop body without spill, no
   change needed. Document the asm output in the packet artifacts.
3. If the compiler bails to a generic loop with bookkeeping
   overhead, transliterate pgvectorscale's match-arm-per-length
   shape and re-verify.
4. Bench: a microbench that calls `hamming_xor_popcount` N=10M times
   on 24-u64 inputs, confirming throughput is at the SIMD-popcount
   floor (~1 cycle per u64 on Skylake-class hardware).

**Files**:
- `src/am/ec_diskann/scan_query.rs` — only if the asm verification
  flags a real codegen issue.
- `benches/criterion/` — add a microbench for the hot path.

### 3. GUC end-state

The merge-readiness review left this open. Decision (per user
direction in 11099 feedback): **keep the GUC as a real production
emergency-rollback knob.**

Concrete steps:

- **Update the doc string** at `src/am/ec_diskann/options.rs:124-127`
  from "Diagnostic override used for Task 29a A/B measurement" to
  something like "Per-session DiskANN scan prefilter override.
  Defaults to `auto` which uses the binary sidecar when persisted
  and falls back to grouped-PQ otherwise. Set to `grouped_pq` to
  force the legacy prefilter for emergency rollback if the binary
  sidecar misbehaves on a specific corpus." The string is what users
  see via `\dc ec_diskann.prefilter_kind` and what shows up in
  `pg_settings`.
- **Add a `pg_test_ec_diskann_prefilter_kind_override_changes_result_quality`**
  test mirroring `test_ec_diskann_session_list_size_override_changes_scan_width`
  at `routine.rs:1980`. Build a small fixture, run a query under
  `Auto` and `GroupedPq`, assert at least one differing result
  (proving the GUC actually toggles behavior). Don't assert on which
  is "better" — just on switching.

### 4. Code-shape cleanup

Two items flagged in the merge-readiness review. Both small.

**a. Extract the prefilter-selection helper.** `routine.rs:598-683`
has the `use_binary_sidecar` decision, the conditional codebook
read, and a duplicated `vamana_scan_with` call (one arm per
prefilter). Extract to:

```rust
enum PreparedPrefilter {
    BinarySidecar { query_words: Vec<u64> },
    GroupedPq { query_lut: Vec<f32>, codebooks: Vec<f32>, group_count: usize },
}

fn prepare_prefilter(opaque: &DiskannScanOpaque, raw_query: &[f32], …)
    -> Result<PreparedPrefilter, String> { … }
```

Then a single `vamana_scan_with` call with the prefilter closure
matched off `PreparedPrefilter`. Reduces the duplicated rerank
closure body.

**b. Fold the `peek_next_active` / `pop_next_active` redundancy.**
`scan.rs:371-400`. The peek skims stale entries; the pop calls peek
again. Either inline the early-stop check around a single pop, or
return both the candidate and a "remove" closure from peek. Trivial
diff, makes the logic linear.

Neither of these is a behavior change — both are pure
extract/refactor. Keep them in a single commit so the diff is easy
to verify.

## Out of scope (deliberate)

- **Removing grouped-PQ from DiskANN entirely.** Grouped-PQ stays
  for vacuum (after item 1), for the GUC fallback (after item 3
  decision), and for `ec_hnsw` / `ec_ivf` which are independent.
  The "no deprecated names" rule applies to *unused* names; this
  one is still used.
- **Removing `tuple.search_code` / `search_code_len` from the tuple
  layout.** Vacuum still depends on it via grouped-PQ. If we ever
  remove grouped-PQ from DiskANN entirely, this becomes a separate
  tuple-format change with its own rebuild path.
- **Removing the `grouped_codebook_head` metadata field.** Same
  reason — vacuum reads it.
- **Build performance.** That's Task 29c.

## Validation gate

- All existing `cargo test` passes (28 in `am::ec_diskann::scan` + the rest).
- All existing `cargo pgrx test pg18 test_ec_diskann_*` passes
  (currently 19/19 at HEAD).
- The new vacuum-repair recall measurement shows no regression vs
  the pre-vacuum recall on the same corpus.
- The new `pg_test_ec_diskann_prefilter_kind_override_*` passes.
- `cargo asm` artifact attached to the packet showing the
  popcount-loop codegen.
- `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
  stays clean.

## Acceptance criteria

- One review packet documenting items 1–4 with packet-local logs
  and asm artifacts.
- Vacuum repair under the sidecar prefilter on real-10k matches the
  scan-path recall floor (≥ 0.99 post-vacuum).
- GUC documentation reflects production rollback intent and is
  testable from SQL.
- The `routine.rs amrescan` and vacuum sites share the prefilter
  selection helper; they cannot drift independently in a future
  change.

## Estimated size

Roughly 3–5 days. Item 1 (vacuum sidecar) is the largest at maybe
2 days including the recall measurement. Items 2–4 are 1 day each.
