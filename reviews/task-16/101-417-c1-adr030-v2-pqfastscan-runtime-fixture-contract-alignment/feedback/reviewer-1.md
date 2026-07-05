## Feedback: PqFastScan Runtime Fixture Contract Alignment

Read the packet, `create_pq_fastscan_binary_runtime_fixture` /
`PQ_FASTSCAN_BINARY_RUNTIME_WORD_COUNT` / `pq_fastscan_binary_runtime_query`
in `src/lib.rs`, the new `assert_f32_close` helper, and the
`grouped_binary_traversal_score_enabled` tightening in
`src/am/scan.rs`.

### What's right

- **Resolves the concern I raised on packet `404`.** Earlier
  feedback flagged that the 16-row/16-dim fixture used for
  rerank tests may have been degenerately small enough that no
  binary sidecar was built, so the binary runtime-settings/profile
  tests were de-facto testing a fallback path. This packet adds
  a genuinely binary-capable fixture (`create_pq_fastscan_binary_
  runtime_fixture` with computed `PQ_FASTSCAN_BINARY_RUNTIME_WORD_COUNT
  = (RECALL_DIM + 63) / 64`) and routes the binary tests through
  it. Now binary coverage is actually binary.
- **`Some(1)` was a bug, not a test simplification.**
  Hard-coding `binary_word_count = 1` masked the fact that the
  real production layout has multiple binary words. The packet
  replaces it with `((dim + 63) / 64)` computed from the fixture
  dimension — the test now matches what the runtime actually
  produces instead of what a 16-dim test happened to land on.
- **Row-ids by observed self-rank instead of hard-coded.** The
  round-trip/vacuum tests no longer assume `id = 1` or `id = 17`
  ranks first. That is a real fix — the old tests would silently
  pass or fail based on fixture-generation-seed luck, not on the
  invariant they actually cared about (round-trip preserves the
  top-ranked row's identity).
- **`grouped_binary_traversal_score_enabled` tightened to only
  fire on `GraphStorageDescriptor::PqFastScan(_)`.** Previously
  a future `turboquant` extension that happened to carry a
  binary-shaped sidecar would have accidentally activated binary
  traversal. The tightening closes that latent footgun while
  moving no runtime behavior on current storage formats.
- **`assert_f32_close` replaces strict equality on source-backed
  exact scores.** Exact `==` was always fragile for floating-
  point scores even when both lanes use the "same" arithmetic —
  any reordering of adds changes the last bit. A small tolerance
  at the operator-facing surface is the right contract.
- **Live-window tests no longer require movement where the
  simulation keeps the same order.** That was a bug in the old
  test — it asserted "larger window changes order" as if that
  were a contract, when the real contract is "larger window
  cannot lose correctness." Fixed.

### Concerns

1. **Big packet with many orthogonal fixes.** Eight concrete
   change items: new fixture, exact-score derivation, tolerance
   helper, live-window semantics, row-id selection, fixture
   reordering for rerank parity, binary-traversal tightening.
   Any of these could have been its own packet. A single rebase
   conflict on any one of them costs the whole packet. For a
   pre-merge alignment push this is probably the right
   trade-off, but it does mean the merge reviewer has to read
   all eight changes independently.
2. **`assert_f32_close` tolerance value not named in packet
   body.** What's the epsilon? `f32::EPSILON`? `1e-6`? `1e-4`?
   The choice matters: too loose and a real regression slips
   through, too tight and floating-point reordering produces
   flakes. Worth naming in the packet body (and ideally as a
   named constant in the source, not a magic number per call-
   site).
3. **"Old tests were asserting stale expectations" is a big
   claim.** The packet frames the old tests as lagging the code,
   which is the right framing if and only if the code was the
   intended contract. But in some cases the old test may have
   been the intended contract and the code drifted. For each of
   the eight changes, the direction — "code was right, test was
   stale" vs "code drifted, test was right and is now wrong" —
   should be named explicitly. The packet assumes the first
   direction throughout; if any case is the second direction,
   this packet silently codifies a regression.
4. **Fixture reordering for rerank-parity coverage.** The
   packet says "reordered the small source-backed fixture used
   by the rerank-parity coverage so its expected ordering matches
   the current source-backed runtime behavior." That phrasing is
   concerning — if the test was asserting ordering and the code's
   ordering changed, the *test* should not bend to match the
   code without a clear reason why the code's new ordering is
   correct. Worth naming the specific ordering invariant that
   moved and why.
5. **Binary-traversal tightening is a runtime-behavior change.**
   The other seven items are test/fixture work; `grouped_binary_
   traversal_score_enabled` tightening at `src/am/scan.rs` is an
   AM behavior change, even if its observable effect is zero on
   current formats. For clean merge story, AM-logic changes
   should travel separately from test-alignment changes. Not
   fatal, but worth noting.

### Observation

Most of this is right, but it is the highest-risk packet in the
418-packet local batch because it changes both tests and (in one
place) runtime. The "code was right, test was stale" framing is
the correct default only if each case was individually verified
— with eight changes in one packet, the reviewer has to take
that on faith. Breaking apart the binary-traversal tightening
into its own packet would keep the test-alignment diff cleaner
and the merge story more mechanical.
