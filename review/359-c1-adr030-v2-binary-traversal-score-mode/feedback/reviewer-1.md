## Feedback: ADR-030 v2 Binary Traversal Score Mode

Read `GroupedTraversalScoreMode` enum at `scan.rs:440`, the resolve
function at `scan.rs:842`, the amrescan wiring at `scan.rs:704-710`,
and the binary traversal scorer that calls through to
`score_binary_sign_words_no_qjl_4bit` at `scan.rs:1737`.

### What's right

- **This is the pivot the branch needed.** 355 showed the
  grouped-built graph is fine. 356-358 showed no cheap exact-rescue
  seam works. This packet asks "what if we replace the grouped-PQ
  traversal score entirely with the binary sign score that's already
  on-page?" and the answer is a 14.6-point recall lift at 50k
  (0.674 → 0.820) and an 11.1-point lift at 10k (0.815 → 0.926).
  That's the biggest quality step on this branch since 355.
- **Score mode resolved once per amrescan.** Same structural pattern
  as 353 and 355 — the mode is stored on `TqScanOpaque` and
  consulted on the hot path as an enum comparison, not an env read.
  Right structure.
- **Reuses the binary query that was already prepared for the
  rejector path.** The note at line 94-96 ("prepare the binary sign
  query on the no-QJL 4-bit lane, even if
  `tqhnsw.disable_binary_prefilter` would otherwise suppress the
  older rejector path") is exactly right — the query is already
  prepared, so binary-traversal mode just promotes it from rejector
  to primary scorer. No new query-prep path.
- **Reuses the already-computed binary rejector score on the
  successor path.** Per the implementation note at lines 105-107:
  grouped-PQ work is out of the loop in binary mode, confirmed by
  the hot-path counter table at line 184-187 showing `grouped approx
  calls = 0.0`, `grouped exact calls = 0.0`, `candidate score us =
  16.1`. That's three orders of magnitude cheaper than the 757us /
  3785us / 6498us budget=4/8 numbers from 357.
- **Explicit rejection when binary lane is unavailable.** Good. The
  grouped-v2 on-disk format currently includes the binary sidecar,
  but defensive checks at amrescan prevent silent fallback to a
  degenerate path.
- **Follow-up note about "binary + exact budget = 0.114" is the
  right kind of negative result to surface.** Lines 218-237 flag
  that the existing budgeted-exact path mixes score scales badly,
  and explicitly scope the current win to "plain binary traversal"
  with budget disabled. That keeps the main claim tight.
- **Scratch install helper addresses the pg_config footgun.** `cargo
  pgrx install` falling back to system `pg_config` (pg14 on this
  machine) would silently produce an extension built against the
  wrong Postgres ABI. `scripts/install_adr030_pg17_pg_test.sh`
  forcing the pg17 path is the right kind of operational guardrail
  to put in the repo — once it matters, it matters at every future
  install.

### Concerns

1. **Local linker failure on the required test commands is a real
   validation gap.** The packet is explicit about it (lines 128-137)
   and the failure mode is a pre-existing workstation issue. But
   that means `cargo test` and `cargo pgrx test pg17` — the two
   required checkpoints — didn't run on the final code. `cargo
   check --tests` and clippy pass, but those don't run the pg proofs
   that this packet claims.

   Specifically: lines 109-113 of the request enumerate pg coverage
   for "runtime settings reflecting grouped_scan_score_mode",
   "invalid grouped traversal score-mode env rejection", and
   "grouped binary mode emitting results while leaving grouped-PQ
   and grouped exact traversal counters at zero." The last one is
   the load-bearing test — it's what proves binary mode doesn't
   accidentally fall back to grouped-PQ.

   That test passed in a pre-commit run somewhere, but not in the
   required checkpoint. For a packet with this much weight on the
   branch trajectory, that's a meaningful gap. Options:
   - get the linker issue fixed before the next packet so future
     pg tests run clean
   - or treat subsequent binary-lane claims as provisional until the
     pg proof runs
   - at minimum, document the linker diagnosis in a packet so the
     next reader isn't reinventing it

2. **`mean abs score error = 464.24554` at 50k and `784.5221` at
   10k.** The packet explains this as "raw binary sign scores live
   on a different numeric scale than the exact `<#>` score" (line
   180, 209). That's the right read qualitatively, but it means the
   emitted comparison surface (which compares approximate to exact)
   now produces numbers that can't be compared cross-mode anymore.
   Two downstream effects worth naming:
   - any diagnostic that averages or thresholds score error across
     modes will be meaningless in binary mode.
   - the "exact-quantized Recall@10" ceiling is still valid (it's
     rank-based), but score-difference metrics aren't.
   Worth a note on the external-summary surface that score-error
   columns are mode-dependent.

3. **Binary score's Spearman@10 is materially worse than scalar's
   (0.58 vs 0.76 at 50k, 0.84 vs 0.93 at 10k) even though Recall@10
   is close.** The same pattern as 355 exact-traversal: top-10
   membership recovers, but within-top-10 ordering is worse than
   scalar. This is actually expected — binary sign is low-precision
   by design — and it means binary-mode grouped-v2 is a good
   *candidate-selection* signal paired with a cold-rerank output
   score, not a good output-ranking signal on its own. The existing
   window=16 live rerank partially addresses this. Worth widening
   the window sweep on the binary lane specifically (packet 360
   does exactly this).

4. **The 50k binary gap to scalar is still 0.07 Recall@10.** Lines
   254-255 call this out. At this point the branch has a viable
   path — binary traversal + larger rerank window — but not yet a
   ship candidate. The next two packets (360 widening window, 361
   deterministic builds) are what close that gap.

5. **`score_binary_sign_words_no_qjl_4bit` is negated at line 1737**
   (`-quantizer.score_binary_sign_words_no_qjl_4bit(...)`). The
   frontier scheduler treats lower score = better. The rest of the
   grouped path is consistent with this convention, so the sign
   flip is correct — but it's worth a one-line comment at the scoring
   call site so a future refactor doesn't accidentally drop the
   negation and quietly invert candidate ranking. Grepping for all
   four callers of the quantizer method, this packet adds two (the
   entry scorer and the successor path); both are negated
   consistently. So the invariant holds, but it's implicit. Easy to
   regress.

### Observation

Three meta-points about this packet's place in the branch:

1. **Pivot is bigger than it reads.** The request frames this as
   "binary mode" but structurally it's "the grouped-PQ traversal
   score is not the right signal for candidate selection, use the
   binary sign instead." That reframe replaces a substantial part of
   ADR-030's original candidate-scoring design with a different
   signal already present on-page. It's the kind of design change
   that would normally warrant an ADR amendment — at least a note
   appended to ADR-030 saying "the binary sign sidecar is promoted
   from rejector to optional primary traversal scorer under the v2
   gate."
2. **The exact-traversal path (355) is now diagnostic only.** Once
   binary mode is the candidate-selection signal, exact traversal's
   role is "upper bound of what candidate-selection quality can
   achieve on this graph." That's still useful for diagnosing
   regressions, but it's not an operating point. Worth naming
   explicitly in the next packet so the exact-traversal gate
   doesn't accumulate cruft around production claims.
3. **The branch narrative is now consistent.** grouped-v2 is a
   storage format that's competitive with scalar when traversed
   with the right signal. The right signal on this corpus is binary
   sign, not grouped-PQ. The grouped-PQ sidecar becomes output-rerank
   support rather than the primary traversal score.

### Measurement gap still open

- corpus-scale recall at 50k: partial — 0.820 binary vs 0.890
  scalar is a ~7pt gap. Getting to parity is the open question for
  packet 360 (wider rerank window) and 361 (deterministic build).
- pg proof of "binary mode leaves grouped-PQ counters at zero":
  claimed passing but the required checkpoint wasn't green on this
  workstation. See concern #1.
- no latency claim at planner-facing level yet, which is right —
  that comes in 360 once the launcher can target the right index.
