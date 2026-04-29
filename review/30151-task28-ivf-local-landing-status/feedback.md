---
reviewer: opus (main-conversation)
date: 2026-04-29
head: 495b6670
scope: cumulative review of 30117–30151, follow-up to feedback at 30106 and 30116
context: 990k/1M benches deferred to production-class hardware per packet 30150
---

# Task 28 IVF coder-1 progress review (round 3)

This round reads the code and packets landed since 30116 and assesses the
local landing posture declared in 30151. The previous feedback at
`review/30116-task28-ivf-pqfastscan-bound-prune-smoke/feedback.md` left five
new code findings (G1–G5) and three open gates (A3, A9, A10).

## Status of prior feedback items

| ID | Issue | Status |
|---|---|---|
| G1 | `running_top` maintained for non-PQ-FastScan profiles | **Closed** by `526971ca`: `quantizer.uses_score_bound_pruning()` gates allocation; the heap is now PQ-FastScan only. |
| G2 | Bound threshold lags on Occupied-better duplicate updates | Acknowledged; not changed. Acceptable — safe under-prune; revisit only if a profile shows it matters. |
| G3 | Missing comment on `consume_live_tid_budget(heap_tid_count == 0)` | **Closed** by `526971ca` with the requested invariant comment. |
| G4 | A7 suffix-max state at high group counts not measured | Not measured, but the local 990k/high-dim deferral in 30150 makes this a benchmark-environment item, not a local landing blocker. |
| G5 | Empty packet directories | **Closed** by `526971ca` (rmdir-only — directories not in tree). |

Four of five closed; G2 deliberately deferred and G4 covered by the 30150
deferral envelope. Good cycle.

## Merge-gate state at 495b6670

| gate | status | evidence |
|---|---|---|
| A1 | landed | 30076 |
| A2 | closed | 30079, 30109, 30129 |
| A3 | **closed for the local v1 claim** | 30139 (diagnostic) → 30140 (list-segregated build) → 30141 (100k same-slice 0% growth) → 30142 (rotating-window churn flat at slack=50). Default `posting_slack_percent=0` keeps the build size unchanged. |
| A4, A5 | landed | 30102 |
| A6 | landed (mixed-predicate caveat) | 30077 |
| A7 | closed for PQ-FastScan path | 30115, 30116, 30117, 30137, 30138 |
| A8 | landed in correctness; **regressed against A5 for RaBitQ** — see H4 | 30081, 30082, 30107 alias |
| A9 | local lane closed; product-class run deferred | 30126, 30130–30135, 30146 (HNSW build budget exceeded), 30150 (deferral) |
| A10 | **conditional on H4 fix** — RaBitQ rows in 30145 measure a wiring bug, not the quantizer | 30097, 30137, 30143, 30144, 30145 |

The local-landing posture in 30151 is defensible. The gate items where
production-class hardware is the bottleneck (A9 1M HNSW reference, fresh 990k
exact recall fills, cold/warm cache claims) are explicitly scoped out via the
30150 deferral, which is the right move.

## Code review of the round-3 commits

### Strong work

- **`4e568d22` list-segregated build.** Clean change: a generic
  `DataPageChain::start_new_page_if_current_has_tuples()` helper plus three
  call sites at the right boundaries (centroid→posting, between non-empty
  posting lists, posting→directory). Idempotent (no-op when the current page
  is empty). Page-level test asserts no shared posting blocks across lists.
  The 30139→30140 result (cross-list blocks 21→0 at n64, index size +5.9%)
  is exactly the expected tradeoff and matches the diagnostic recommendation.

- **`419a0713` posting slack.** Pure additive reloption with `default=0`,
  `max=1000`. The integer ceiling math
  `(posting_pages * slack_percent + 99) / 100` is right. Default behavior
  is unchanged for existing users; explicit churn workloads can opt in.
  The 30141 → 30142 progression is honest about the cost: at slack=50,
  n32 build size grew from 9.06 MB to 13.62 MB but stayed flat through 10
  rotating-window cycles where slack=0 grew 23.8%.

- **`526971ca` G1/G3 cleanups.** The `uses_score_bound_pruning()`
  predicate is the right shape — the bound-prune contract lives on the
  quantizer, the scan path gates on it, no string comparison reintroduced.

- **`06701988` page-ownership diagnostic.** Adds
  `ec_ivf_index_page_ownership(index_oid)` as an admin diagnostic. This is
  the kind of investigation seam I would have asked for and the team built
  it before churn closure — credit due. It made the 30139/30140 narrative
  pageinspect-grounded instead of speculative.

- **A7 closure on 10k/25k (30137).** EXPLAIN counters
  (`postings_visited`, `postings_scored`, `postings_pruned_by_bound`)
  prove the bound is doing measurable work — 5285/7578 visits pruned at
  10k, 16256/19750 at 25k. That converts "smoke says faster" into
  "instrumented says we know why."

- **A10 matrix at 30145.** Three quantizers, recall@10/100, p50/p95/p99,
  HWM, index size, all on matched shape. RaBitQ rows correctly noted as
  bounded (queries-limit=20, iterations=10) because measured per-query
  latency is multi-second.

### Concerns to address before merge

#### H1. `ensure_heap_tid_absent_in_list` removal in `63e3eaf3` lacks an in-code rationale

The production live-insert duplicate scan was removed. The packet
justifies it as "PostgreSQL heap TIDs are unique for normal index inserts."
That is correct under PG's normal contract: `aminsert` is called with a
fresh heap TID for INSERT and non-HOT UPDATE; vacuum has removed any
previously-indexed entries before the heap line pointer can be reused.
The existing `debug_ec_ivf_validate_no_duplicate_heap_tid` helper still
covers the corruption-check path.

That said, this is an invariant that is not obvious from reading
`insert.rs` cold. A future reader removing the test helper or refactoring
vacuum could break it silently. Two options:

- Add a short `// invariant:` comment at the call site in
  `insert_into_trained_index` referencing the PG `aminsert`/vacuum
  contract.
- Keep the helper but only run it under a debug build feature (`#[cfg]`
  gate it out of release rather than deleting it). I do not recommend
  this — the current shape is cleaner — but if the team later finds a
  vacuum/insert race they can't reproduce, they will want it back.

The comment alone is enough to land. The pg_test duplicate-validation
helper covers the regression net.

#### H2. `posting_slack_percent` not surfaced on existing surfaces' admin snapshot

`relation_options` reads it; `index_admin_snapshot` does not surface it.
Operators tuning churn workloads will need to rebuild and verify the
slack took effect. The `effective_*` snapshot pattern used for `nprobe`
and `rerank_width` is a good model. Add:

- `relation_posting_slack_percent` in the admin snapshot, alongside the
  existing relation reloption fields.

This is purely additive, mechanical, and gives users a way to check
whether their build actually reserved slack without resorting to
`ec_ivf_index_page_ownership`.

#### H3. The `+1` in `append_empty_pages(slack_pages + 1)` is load-bearing and not commented

`build.rs` near the new slack code:

```rust
let (_, slack_tail) = data_pages
    .append_empty_pages(slack_pages + 1)
    .expect("positive slack should append pages");
directory_tail_blocks_by_list[list_id] = Some(
    slack_tail
        .checked_sub(1)
        .ok_or_else(|| "posting slack page underflow".to_owned())?,
);
```

The `+1` reserves a separator page so the next list's
`start_new_page_if_current_has_tuples` doesn't see the last slack page as
empty and write into it. The `-1` puts the directory tail one block
before that separator. Without the separator, the next list's first
posting would land in the previous list's slack range and corrupt the
"no cross-list pages" guarantee.

This is correct, but if you remove the separator later in a refactor it
will silently regress to the 30139 cross-list-pages behavior. Add a
two-line comment:

```rust
// +1 reserves a separator page after the slack range so the next list's
// first insert does not reuse this list's last slack page; the directory
// tail is the last in-range slack block (slack_tail - 1).
```

#### H4. RaBitQ A10 latency is an IVF wiring bug, not a quantizer-quality bug — fix before merge

**Updating this finding after reading the code.** The original framing
("RaBitQ kernel is slow, document it") was wrong. The 30145 RaBitQ
numbers — p50 1947 ms (10k) and 4973 ms (25k) at matched shape — are
caused by a regression of A5 in the A8 wiring, not by the RaBitQ
quantizer itself.

**The bug.** `src/am/ec_ivf/quantizer.rs:245`:

```rust
(IvfQuantizerProfile::RaBitQ, IvfPreparedQuery::RaBitQ(prepared_query)) => {
    let quantizer = self.rabitq_quantizer()?;     // rebuilt every posting
    let _ = gamma;
    Ok(quantizer.estimate_ip(prepared_query, payload).estimate)
}
```

`self.rabitq_quantizer()` (`quantizer.rs:332`) calls
`RaBitQQuantizer::with_seeded_srht_bits(dim, seed, bits)`, which
allocates `Arc::new(SrhtRotation::with_seed(dim, seed))`.
`SrhtRotation::with_seed` (`src/quant/rabitq.rs:141`) calls
`rotation::sign_vector(transform_dim, seed)` — builds the SRHT sign
vector from scratch. There is no `OnceLock` cache.

The TurboQuant arm at `quantizer.rs:227` uses `ProdQuantizer::cached(...)`,
which A5 verified is process-cached (`pg_test_ec_ivf_rescan_reuses_cached_prod_quantizer`).
The RaBitQ arm got the same audit *skipped* when A8 added two new
dispatch arms. A5 was closed when only TurboQuant was in scope; A8
shipped without re-running the audit.

At `nlists=64, nprobe=48` on 25k corpus, ~75% of postings are scored
per query → ≈18k full quantizer reconstructions per query. That is
what the multi-second p50 is paying for, not the kernel.

**The reconstruction isn't even needed at score time.**
`RaBitQQuantizer::estimate_ip` (`src/quant/rabitq.rs:405`) only reads
two fields from `self`:

```rust
estimate_ip_impl(
    &prepared.query_rotated,
    prepared.query_norm,
    self.dimensions,        // already on PreparedEstimator
    self.bits_per_dim,      // crate::DEFAULT_QUANT_BITS — constant
    code,
)
```

`prepared.query_rotated` already has the SRHT rotation applied at
`prepare_estimator` time. The hot path doesn't need a quantizer at all.

**Two acceptable fixes.** Pick one:

1. Mirror `ProdQuantizer::cached` for RaBitQ. Add a process-level
   `OnceLock<Mutex<HashMap<(usize, u64, u8), Arc<RaBitQQuantizer>>>>`
   cache keyed on `(dimensions, seed, bits)` and have
   `IvfQuantizer::rabitq_quantizer()` consult it. This is the
   minimum-change fix and matches the existing pattern.
2. Skip the construction entirely on the score path. Either expose
   a free function `crate::quant::rabitq::estimate_ip_with_bits(
   prepared, bits, code) -> DistanceEstimate` (or move
   `estimate_ip_impl` to a public free function), or move
   `estimate_ip` onto `PreparedEstimator` carrying its own
   `bits_per_dim`. Then the IVF arm reads only the prepared estimator
   and never builds a `RaBitQQuantizer` per posting.

Option 2 is structurally cleaner — there is no reason the IVF score
path should hold a quantizer reference if the prepared state already
carries everything `estimate_ip` reads. Option 1 is the lower-risk
back-port if you want to land the fix without touching `quant::rabitq`.

A regression test analogous to
`pg_test_ec_ivf_rescan_reuses_cached_prod_quantizer` should accompany
the fix: build a RaBitQ index, run two scans, assert that no per-
score-call quantizer rebuild is observable (e.g., via a debug counter
on the construction path, or by asserting `Arc::strong_count` if you
take option 1).

**Implication for A10.** The 30145 RaBitQ row is measuring this bug,
not the quantizer. Re-run the A10 matrix RaBitQ rows after the fix
lands. The recommendation paragraph in 30145 ("RaBitQ as a future
optimization path, not the current default candidate") should also be
revisited — the bound the recommendation rests on is a wiring artifact.

This **is** a merge-blocker. The local-landing claim in 30151 says A8
and A10 are done; both rest on RaBitQ being a real, supported variant
that is comparable on the same axes as the others. A 25–40× slowdown
caused by per-posting reconstruction is not "real, supported" — it
is shipped-but-broken.

#### H5. The 30141 100k 10-cycle "0.0% growth" claim is on the same-slice churn shape, not the rotating-window shape

30141 itself flags this: "the rotating-window diagnostic is still not
closed" with same data, 10 cycles, n32 grew 23.8% (9.04 → 11.20 MB).
30142 closes the rotating-window result by adding `posting_slack=50`,
which keeps it flat at the cost of 50% larger build.

The local-landing claim in 30151 reads "A3 done for local v1 claim,"
which is fair, but the explicit caveat — *flat-on-rotating-window
requires opt-in slack, not the default* — should be the headline of the
A3 closure, not a footnote. Either:

- Update 30151 (or add a short follow-up packet) so the A3 row says
  "default rotating-window churn grows 24%; flat behavior requires
  explicit `posting_slack_percent`," or
- Make `posting_slack_percent` default to a small non-zero value (10?)
  so the default user is closer to the flat curve. I do not recommend
  changing the default in this branch — the doc is enough.

The current write-up risks reading more confidently than the data
supports. The data is fine; the framing needs sharpening.

## How to reach final closure for the local-landing branch

In priority order:

1. **H4 (RaBitQ per-posting reconstruction)** — code fix in
   `src/am/ec_ivf/quantizer.rs:245` plus a regression test in the
   shape of A5's `pg_test_ec_ivf_rescan_reuses_cached_prod_quantizer`.
   This is a merge blocker. Pick option 1 (cache mirror) for the
   smallest diff or option 2 (skip construction) for the cleaner
   structure; either is acceptable.
2. **A10 RaBitQ re-measurement** — rerun the 10k/25k matched-shape
   rows after H4 lands and update 30145. The recommendation paragraph
   should be regrounded on the post-fix numbers; if RaBitQ becomes
   competitive at recall, A10's bias-honesty wording requires saying
   so plainly.
3. **H3** (slack `+1` comment) — 2 lines, no test impact.
4. **H1** (insert dedup invariant comment) — 2 lines, no test impact.
5. **H2** (admin snapshot exposes `posting_slack_percent`) — additive,
   one-line struct field plus the snapshot read site. Add one PG18 test
   asserting the field reflects the reloption.
6. **H5** (sharpen 30151's A3 row to call out the rotating-window/slack
   tradeoff) — packet-only.

After those six, the local-landing claim in 30151 is honest, and the
deferred items (1M HNSW reference, fresh 990k recall, cold/warm cache,
A7 high-dim suffix size) belong to a follow-on task scoped to
production-class hardware, *not* to this merge gate.

## On the 990k / 1M deferral itself

The decision in 30150 to defer fresh 990k exact-recall fills is
defensible:

- The 30149 recall job stalled at ≈22 minutes still fetching the source
  matrix on this hardware. That is harness-scale, not IVF-code-scale.
- 30146 showed even the 100k HNSW reference build can blow past a local
  time budget and required a process kill.
- Existing 990k IVF packets (30130/30132/30133/30135/30136) provide
  directional evidence — they're not abandoned, just not extended.

Two things to make the deferral concrete:

- **Open a follow-on task** numbered after 28, with a one-liner per
  deferred item: `(a) 1M HNSW reference comparison, (b) fresh 990k
  exact recall fill, (c) cold/warm cache rows for A9/A10, (d) A7
  high-dim suffix-max state regression check`. The 30151 packet
  references these but doesn't pin them to a numbered task. Without
  a task they will be lost.
- **Lock the recall-truth-cache file format.** 30147–30149 added
  `--truth-cache-file` and partial top-k extraction; the next-machine
  run will want to consume an existing cached truth file, not regenerate
  it. Make sure the file format is stable (a one-line schema doc in the
  packet 30148/30149 area is enough).

## What is strong about this round

- The A3 narrative is the right shape: diagnose first (30139), implement
  the narrowest fix (30140), measure (30141), discover the rotating-
  window gap, propose and measure the explicit tradeoff (30142). The
  team kept defaults conservative (`posting_slack_percent=0`).
- The A7 closure carried the explain-counter instrumentation forward,
  so the 30137 result is grounded ("the bound did this much pruning"),
  not asserted.
- A10's RaBitQ row is honest about latency. Refusing to drop RaBitQ
  from the matrix because it loses is the right tone for the gate.
- 30150 names the deferral instead of pretending the gap doesn't
  exist. The 30151 status packet is appropriately scoped to local.
- Five of the prior feedback's five code findings closed in the next
  commits, including the rmdir of empty packet dirs. Tight loop.
