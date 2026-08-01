# Task 210 review request: zero-byte membership head + derived shard ordinal (round 2)

- Branch: `task-203-ec-distann-conformance`
- Commits under review: `35c7f3c3b` (membership-only head as a bounded
  state-row blob, zero sample/graph rows), `4070ff6cb` (shard ordinal derived
  from members on owner and replica paths), `a5bd2c33d` (allowlist deleted +
  `--local-head` control support), `0e4e28641` (reader learns the `control`
  storage class).
- Evidence: `artifacts/manifest.md` + `artifacts/run/results.jsonl`
  (9 steps × 10/50/100k).

## Both round-2 P1s, resolved with evidence

1. **The explicit zero-byte gate is met.** The shipped default persists the
   head as a single bounded membership blob on the state row; the sample and
   graph relations hold zero rows. Candidate conformance rows:
   `coordinator_resident_unsharded_bytes=0`,
   `outstanding_distribution_gap=none`, conforming, decision-eligible, at
   every scale. The allowlist is deleted — the ratchet you asked for: a
   reappearing head relation is a hard violation attributed `unowned`, and
   the legacy shape (kept measurable via `--local-head`) now honestly
   evaluates nonconforming, which also means NFR-022 forbids it from being a
   *control*; it rides as the context referent.
2. **Replica materialization uses the shard's true ordinal.**
   `shard_owner_ordinal()` derives it from the member ids (uniform ownership
   validated; imported rows filtered by it), and both the shard cache key and
   the graph seed use it on owner and replica paths. The predicted
   consequences confirmed in the A/B: the 50k/100k replica recall deficit is
   gone (100k now 0.9290, above the referent's 0.9275) and the +25% replica
   latency outlier collapsed to parity (38.40 vs 39.30 ms) — the
   per-serving-node topology and unstable cache identity were exactly the
   cost you inferred.

Replica serving remains provably active (`head_replica_shards_served`
29/33/32, fallbacks 0). Recall is identical-or-better for the default at
every scale; the default's latency cost against the non-conforming referent
(+8.6% 10k, −0.5% 50k, +5.1% 100k) stays recorded under 003a question 2.

Request open.
