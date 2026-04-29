---
reviewer: opus (main-conversation)
date: 2026-04-29
head: 20fc7504 (fix at 91964193)
scope: review of H4 closure plus the H1/H2/H3 follow-on cleanups landed in the same commit
---

# Task 28 IVF coder-1 progress review (round 4)

This packet records the H4 fix and the H1/H2/H3 cleanups from feedback at
`review/30151-task28-ivf-local-landing-status/feedback.md`. Reading the
fix, the regression test, and the post-fix RaBitQ measurements:

## Outcome

**H4 is closed.** The coder picked option 2 (the structurally cleaner
fix): `bits_per_dim` now lives on `PreparedEstimator`, and
`IvfQuantizer::score_ip_from_parts` for the RaBitQ arm calls
`prepared_query.estimate_ip(payload)` directly with no
`RaBitQQuantizer` reconstruction. `payload_len` likewise switched to
the new `code_len_for(dim, bits)` free function so the length
computation does not allocate either.

Measured impact (from packet 30152 vs 30144 at the same matched shape
`nlists=64, nprobe=48, rerank=heap_f32, rerank_width=750`):

| corpus | broken p50 | fixed p50 | speedup |
|---|---:|---:|---:|
| 10k | 1947.8 ms | 344.2 ms | 5.66× |
| 25k | 4973.0 ms | 775.7 ms | 6.41× |

Recall is unchanged (1.0000 / 1.0000 at recall@10; 0.9930 / 0.9915 at
recall@100), confirming this was purely a wiring fix.

The regression test
`rabitq_dispatch_does_not_rebuild_quantizer_while_scoring` is the
right shape: it pins `seeded_srht_construction_count` at 2 after
encode + prepare, asserts the count does not grow across 8 score
calls. This nets out the original concern from A5 — that the cache
audit was not re-run when A8 added new dispatch arms — by adding a
direct-counter assertion the next review can grep for.

## Other prior findings closed in the same commit

- **H1** (`insert.rs:137`): the four-line invariant comment is
  in. The `aminsert`/VACUUM heap-TID uniqueness contract is now
  documented at the call site, and the `debug_ec_ivf_validate_no_duplicate_heap_tid`
  helper is referenced as the corruption-check path. Closes the H1
  documentation gap.

- **H2**: `relation_posting_slack_percent` is exposed in
  `IndexAdminSnapshot` and read from `relation_options` in
  `index_admin_snapshot`. PG18 admin-snapshot test coverage updated.
  Closes H2.

- **H3** (`build.rs:407–411`): the slack `+1` separator now has the
  comment explaining why removing it would silently regress to
  cross-list page sharing. Closes H3.

## Remaining items

- **H5** (sharpen the 30151 A3 row to call out the
  rotating-window/slack tradeoff). This was a packet-only fix and is
  not addressed by 91964193 / 30152. Still open.

- **A10 RaBitQ row update.** 30152 reports the post-fix RaBitQ
  numbers but does not update the consolidated A10 packet (30145).
  At minimum, 30145 should reference 30152 or be amended so the
  matrix reads with the corrected RaBitQ row instead of the
  pre-fix one. The recommendation paragraph in 30145 ("RaBitQ as a
  future optimization path, not the current default candidate")
  should be re-evaluated against 344 ms / 775 ms p50, not against
  1947 ms / 4973 ms p50. RaBitQ is still slower than PQ-FastScan
  g8 (77 ms / 117 ms p50 on the same shape), so the recommendation
  itself probably does not change — but the *grounds* for keeping
  the recommendation should be the post-fix numbers.

## Code findings remaining

### I1. `encode_source` for RaBitQ still reconstructs per call

`src/am/ec_ivf/quantizer.rs:104`:

```rust
IvfQuantizerProfile::RaBitQ => {
    let quantizer = self.rabitq_quantizer()?;
    Ok((dimensions, 0.0, quantizer.encode_code(source).into_vec()))
}
```

This is not in the same severity class as the H4 score-time
reconstruction — `encode_source` is bounded by row count (once per
build tuple, once per live insert), not by per-query × scored-fraction
× oversample. But the same A5 audit logic applies: a future reader
should not need to know that this construction is "amortized enough"
without it being explicit.

The build path is the lower priority of the two — it amortizes
across the whole bulk-build wall-time. The live insert path is
where this matters: `aminsert` calls `encode_source` once per row
on the hot insert path. At 1536D the SRHT sign vector is large
enough that a per-row reconstruction is not free.

This is **not a merge blocker**. Three options, in priority order:

1. Defer to a follow-on packet measuring the live-insert hot path
   for `quantizer = 'rabitq'` at 1536D against the same
   `ecaz stress ivf-insert` harness used in 30065. If the
   reconstruction shows up in the profile, fix it then.
2. Apply the cache-mirror approach (option 1 from H4) to
   `with_seeded_srht_bits` so all callers benefit. This is more
   conservative and removes the question entirely.
3. Leave it. Build/insert encode is a lower-frequency event and
   defaults are PQ-FastScan / TurboQuant, not RaBitQ. A user
   selecting `quantizer='rabitq'` for build is implicitly opting
   into a non-default profile anyway.

I would lean toward (1) — measure first, decide based on data — to
avoid touching `quant::rabitq` again for a hypothetical hotspot.

### I2. The test counter has a shared-static parallelism caveat

`SEEDED_SRHT_CONSTRUCTION_COUNT` and `SEEDED_SRHT_CONSTRUCTION_COUNT_DIMENSIONS`
are process-global `AtomicUsize` values. Cargo runs unit tests in
parallel by default. The dimension filter
(`SEEDED_SRHT_CONSTRUCTION_COUNT_DIMENSIONS.load == dimensions`)
mostly isolates concurrent tests using different dimensions, but
two tests that both reset the counter for *different* dimensions in
quick succession can race the dimensions store and clobber each
other's expected counts.

In the current tree there is exactly one consumer
(`rabitq_dispatch_does_not_rebuild_quantizer_while_scoring`), so
this is theoretical. If a future test wants to assert construction
counts for a different dimension and runs alongside the existing
test, the assertion can flake.

Two cheap mitigations if you ever add a second consumer:

- Run the construction-count tests with `#[serial_test::serial]`
  and a single global mutex.
- Replace the static atomics with a thread-local counter set up
  through a scope guard. The `with_seeded_srht_bits` fast-path
  can read `THREAD_COUNT.with(|c| c.fetch_add(1))`.

Note for record. Not blocking.

## Verdict on the local-landing branch

Per the gate matrix at 30151 plus this round's closures:

- A1, A2, A4, A5, A6, A7, A8 — landed.
- A3 — closed for the local v1 claim with the rotating-window/slack
  caveat that should be promoted to the headline (H5).
- A8 — *now* genuinely landed: the wiring no longer regresses A5 for
  RaBitQ, and the regression test prevents recurrence.
- A9 — local lane closed, product-class deferred per 30150.
- A10 — needs the small packet update to reference the post-fix
  RaBitQ numbers (or amend 30145 in place).

The branch is in landing-ready posture once H5 (packet-only) and the
A10 packet update land. I1 and I2 are follow-ons.

## What is strong about this round

- Picked option 2 (cleaner structure) over option 1 (cache mirror).
  That is the right call: there is no longer a quantizer reference
  on the hot path at all, which is structurally simpler than caching
  a quantizer that doesn't need to exist.
- The regression test asserts construction count, not just latency.
  A latency assertion would have flaked on hardware variance; a
  construction-count assertion is invariant.
- Three of three small follow-on findings (H1/H2/H3) landed in the
  same commit as H4. Tight loop.
- The 5–6× speedup is exactly in the band the original kernel claim
  predicted (~8 ns/candidate), so the post-fix latency is now
  measuring the kernel and the memory traffic, not the setup.
