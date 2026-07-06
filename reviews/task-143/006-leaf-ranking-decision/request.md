# Task 143 Packet 006 Review Request: Leaf Ranking Decision

## Summary

This packet closes the Task 143 evidence pass with a promote / iterate /
negative decision over the release A/B packets:

- `reviews/task-143/003-release-10k-ab`
- `reviews/task-143/004-release-50k-n1024-ab`
- `reviews/task-143/005-release-100k-n1024-ab`

No new benchmark jobs were run for this packet. The decision cites the release
`ecaz bench suite` artifacts already committed in those packets.

## Decision

- Treat leaf-score-only routing as a positive, release-validated candidate. It
  improves equal-nprobe recall across 10k, 50k, and 100k, and it often improves
  latency at the current higher-probe frontier.
- Do not flip it default-on in this packet. The measured release A/B covers the
  2-level exact-leaf grid (`nlists=128` at 10k, `nlists=1024` at 50k/100k,
  b0, all leaf centroids scored exactly in f32). It does not cover deeper
  hierarchies, larger fan-outs, or approximate leaf scoring where parent
  `path_score` may still carry signal. That coverage gap, not the half-nprobe
  frontier-reanchoring bar, is the conservative default-off rationale.
- Keep route overfetch diagnostic/default-off. Overfetch improves baseline but
  does not beat leaf-only recall at 100k and only marginally beats leaf-only at
  one 50k endpoint.
- Hand the remaining route precision question to Task 144. Route containment
  equals final distinct recall in the source packets, so the remaining gap is
  still route / leaf selection.

## Evidence Highlights

| Fixture | Best Task 143 read |
| --- | --- |
| 10k/n128 | leaf-only nprobe32 reaches recall `1.0000` at p50 `89.706 ms`, beating baseline nprobe64 recall `0.9995` and p50 `174.386 ms`. |
| 50k/n1024 | leaf-only improves baseline recall at every probe; nprobe64 is `0.9475` at `122.661 ms` vs baseline `0.9390` at `128.159 ms`, but leaf-only nprobe32 does not catch baseline nprobe64. |
| 100k/n1024 | leaf-only improves recall at every probe; nprobe96 is `0.9570` at `362.912 ms` vs baseline `0.9300` at `371.433 ms`, but leaf-only nprobe32 does not catch baseline nprobe64. |

See `artifacts/manifest.md` for the detailed decision table and packet-local
source references.

## Review Focus

Please confirm the decision boundary:

- leaf-score-only routing should remain a default-off candidate until broader
  shape coverage or a later frontier-selection task promotes it;
- overfetch should remain default-off;
- the combined `leaf_score_only=on` plus `route_overfetch_multiplier>1.0` cell
  is out of scope for this closeout because the isolated overfetch lever is
  dominated; Task 146 should include the combined cell if it revisits leaf-only
  promotion;
- Task 144 should continue from the route containment evidence rather than
  rerunning this Task 143 matrix.
