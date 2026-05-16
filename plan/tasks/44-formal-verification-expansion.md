# Task 44: Formal Verification Expansion (Kani + Flux on Real Invariants)

Status: **proposed** — replaces Task 34's seed Kani harness and synthetic
Flux harness with proofs over the actual ECAZ invariants the plan
identified.

## Scope

Two formal verification surfaces:

1. **Kani** bounded model checking over wire/layout/arithmetic invariants:
   - `ItemPointer` decode contract (already shipped — Task 34).
   - Tuple alignment arithmetic in `src/storage/page.rs`,
     `src/am/*/page.rs`.
   - Quantizer payload length math (`payload_len`, `mse_bits`).
   - SPIRE leaf V2 segment metadata: segment count vs. byte length,
     payload format vs. column count.
   - Top-k merge ordering: candidate priority queue invariants.
   - Partition routing: hash-to-partition mapping correctness.
   - Remote payload parser rejection behavior (every malformed prefix
     rejected; every well-formed prefix accepted).
2. **Flux** refinement types over dimension / index invariants:
   - `dim` and `bits` constraints across the quantizer API surface.
   - Codebook size = `2^bits` invariant.
   - Page offset / item-index bounds across `src/storage/page.rs`.
   - SPIRE partition id ranges and epoch monotonicity.

## Why

Bounded model checking and refinement types catch a class of bug that no
amount of testing reliably finds: edge-case arithmetic that is only wrong on
inputs not covered by tests, and type-level invariants that are correct in
documented use but unguarded against misuse.

Task 34 landed:

- One Kani proof on `ItemPointer` encode/decode (real coverage, useful).
- A synthetic Flux harness with three fake functions that don't appear in
  ECAZ.

The Flux harness in particular is a "tool installed and passing" result, not
a verification of any real invariant. The Kani harness is real but covers
one type; the plan listed seven invariants, six remain.

## Approach

### Kani

1. **Harnesses follow the lifted-module pattern** (Task 40): pure-Rust
   modules that both production and the Kani harness depend on, included
   via `#[path]` from `hardening/kani/src/lib.rs` or via a workspace dep.
2. **Bounded inputs:** every Kani proof states explicit bounds with
   `kani::assume()` for inputs that would otherwise blow up the SAT solver
   (e.g., `assume(len <= 64)` for variable-length encodes).
3. **Per-invariant proofs:**
   - `kani_tuple_alignment_no_overflow` — for every alignment-padded
     tuple writer, prove that the computed end offset is ≤ page size.
   - `kani_payload_len_matches_unpack` — for every `(dim, bits)` pair,
     prove `payload_len(dim, bits) ==` actual bytes produced.
   - `kani_leaf_v2_meta_segment_count_invariant` — prove segment count
     derived from byte length matches recorded count.
   - `kani_topk_merge_order` — prove the merge of two sorted candidate
     vectors yields a sorted output bounded by k.
   - `kani_partition_routing_total_function` — prove every valid
     `(hash, partition_count)` maps to a defined partition.
   - `kani_remote_parser_rejects_short` — prove every prefix shorter than
     the minimum is rejected.

### Flux

4. **Real refinement annotations.** Replace `hardening/flux/src/lib.rs`'s
   fake functions with Flux annotations on the actual ECAZ public API
   surface where dimension and index bounds matter:
   - `ProdQuantizer::new(dim: usize{0 < dim && dim <= 4096}, bits: u8{2 <= bits && bits <= 8}, ...)`
   - `Codebook::lookup(idx: usize{idx < codebook_size})`
   - `Page::tuple_at(i: usize{i < self.tuple_count})`
5. **Codebook size invariant.** Express `codebook.len() == 1 << bits` as a
   Flux refinement and verify every construction site.
6. **Epoch monotonicity (where Flux supports it).** Annotate the SPIRE
   epoch type so the type system rejects any decrement.
7. **Targeted vs. broad.** Flux is invasive — it requires annotation
   throughout call paths. Pick high-value invariants and accept incremental
   adoption. Do not attempt to annotate the whole codebase.

### Lanes

8. **Make lanes:**
   - `make kani` (existing) — runs all proofs in `hardening/kani/`.
   - `make kani-full` — runs all proofs at deeper unwind bounds for
     longer per-proof budgets, nightly.
   - `make flux` (existing) — runs over the real-annotation set.
   - `make flux-coverage` — reports which annotated functions/types are
     reached by the current Flux pass.

## Validation

- Each Kani proof from the list above completes within a documented
  per-proof budget (default 5 min, harder proofs documented separately).
- Injecting a known arithmetic bug (e.g., off-by-one in `payload_len`)
  produces a Kani counterexample that the harness prints in CI-readable
  form.
- Flux annotations on `ProdQuantizer::new` reject a call with `dim = 0`
  at compile time.
- A removal of a `kani::assume` bound produces either a counterexample or
  a timeout — both are explicit signals, not silent successes.

## Exit Criteria

- All seven Kani invariants listed under "Scope" have a passing proof.
- Flux annotations cover the quantizer construction API, page tuple
  indexing, and at least one SPIRE invariant.
- `docs/hardening.md` documents per-proof budgets and the policy for
  adding new proofs.
- `hardening/flux/` no longer contains synthetic placeholder functions.

## Dependencies

- Task 40 lifts SPIRE coordinator types into pure-Rust modules; some Kani
  proofs depend on that lift.
- Independent of Tasks 36–39, 41–43.
- Pairs naturally with Task 42 (on-disk invariants) — many Kani proofs
  *are* the formal statement of the invariants Task 42 enforces at
  runtime.
