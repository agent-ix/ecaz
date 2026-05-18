## Feedback: ADR-030 v2 Exact Traversal Scan Mode

Read the gate resolution at `scan.rs:861` (`resolve_grouped_exact_traversal_mode`),
the amrescan wiring at `scan.rs:704-730`, and the candidate-scoring dispatch
that feeds exact-payload scoring into the grouped traversal.

### What's right

- **Directly answers the 354 open question.** My 354 feedback asked:
  "measure grouped-v2 recall on the same grouped-built graph but with
  exact scoring at traversal time." This packet is that experiment
  and it produces a clean, decisive answer: `50k @ ef=128` jumps from
  0.674 to 0.878. The grouped-built graph is fine — the grouped-PQ
  traversal score was the main quality loss. That's the single most
  valuable datum in the whole branch to date.
- **Gate resolved once per amrescan, stored on opaque.** Per the
  "Planned Slice" #3 and confirmed in code at `scan.rs:704-718`. No
  per-candidate env reads in the hot path. Right structure.
- **Scalar paths untouched.** The exact traversal gate resolution is
  inside a `matches!(scan_graph_storage, GraphStorageDescriptor::GroupedV2(_))`
  block, so scalar scans cannot be affected even if the env is set
  globally. Same discipline as 353's window-scope gating. Good.
- **Reuses the existing cold-payload rerank path.** The implementation
  loads the grouped cold rerank payload and scores through the shared
  exact rerank scorer that's already used for grouped emitted
  comparison. No duplicate scoring path. Also means the pg test
  claim — "exact-traversal emits the same score it records as the
  comparison sidecar" — is automatically true by construction,
  which is the right invariant to anchor this packet on.
- **Deliberate restraint on planner-facing latency claim.** The
  "intentionally did not add a new planner-facing SQL latency claim"
  paragraph (lines 121-124) is right: the verified SQL launcher can
  silently fall back to the scalar canonical index, so a latency
  number here would have been misleading. This stays on the external
  recall summary lane until the launcher can disambiguate.

### Concerns

1. **The summary table claim "grouped exact traversal mostly closes
   the gap" understates an interesting anomaly.** At 50k @ ef=128,
   grouped exact vs scalar:
   - grouped: 0.8780 / NDCG 0.9198 / Spearman 0.6768 / below_exact=18
   - scalar:  0.8900 / NDCG 0.9289 / Spearman 0.7583 / below_exact=1

   The `below_exact` jump (1 → 18) and the Spearman drop (0.76 → 0.68)
   are both sizable even though top-10 recall nearly matches. That
   means grouped exact-traversal returns *mostly* the right top-10,
   but the ordering within the top-10 (and top-100) drifts more than
   scalar. Two possibilities:
   - the grouped-built graph's top level is still subtly different
     from scalar's — closer-but-not-identical neighborhood structure.
   - the exact traversal path is still gated by window=16 live rerank
     and whatever limits the emitted-set reordering imposes.

   Worth naming explicitly: "grouped exact *recovers top-10 membership*
   but not top-10 *ordering* or Recall@100." Doesn't change the
   qualitative win, but the gap-to-scalar at higher-rank metrics is
   still real.

2. **10k grouped exact matches scalar on Recall@10 but
   exact-quantized ceiling differs (0.9330 vs 0.9310).** That inversion
   is the "exact-quantized is a lossy proxy" pattern from earlier
   feedback showing up again — exact-quantized is using the index's
   own quantized codes as ground truth, and grouped's 4-bit codes
   happen to agree with the actual top-10 more often than scalar's
   8-bit codes do on this corpus. Not a bug, just a reading hazard
   that's now hit twice. Worth capping with a stable disclaimer in
   request/result tables.

3. **No latency measurement in this packet.** I get why (see "right"
   point about the canonical-table planner ambiguity), but the
   natural next reader question is "how much cost does exact
   traversal buy this recall recovery?" The direct harness could have
   answered that locally without the launcher. Packet 356/357 do fill
   it in, but for this packet it means the headline claim
   ("materially changes the diagnosis") can't be sized in cost terms
   on its own. Not a blocker, just a note.

4. **`TQVECTOR_EXPERIMENTAL_ADR030_V2_SCAN_EXACT_TRAVERSAL` is a
   second env next to `_SCAN` and `_SCAN_WINDOW`.** The env-surface
   is getting busy — three envs gated by a hidden build env. As
   noted in 353 feedback, GUC promotion is coming. When that happens,
   nest these under a single `tqvector.adr030_v2.*` namespace instead
   of separate envs for each knob. For now, resolving each once per
   amrescan is the right shape; just worth flagging the sprawl.

### Observation

This is the cleanest diagnostic packet in the recent arc. It asks
exactly the question that was outstanding, implements it as narrowly
as possible, and returns a decisive answer that redirects the whole
investigation. My 354 feedback proposed this experiment as the
"single most valuable follow-on"; this packet landed it and the
result was unambiguous.

The upshot for the branch: stop trying to fix the grouped-built graph
(it's fine) and start replacing the grouped-PQ traversal score with
something better. The exact-traversal result is the upper bound of
what's recoverable through traversal-score quality alone.

### Measurement gap still open

Not for this packet — it was targeted and closed cleanly. The gap
moves: find a cheaper *approximate* traversal score than full exact
that preserves most of this recall lift. Packets 356-359 take up
that thread.
