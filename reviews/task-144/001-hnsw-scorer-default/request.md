# Review request: Task 144 — HNSW int8_approx exact-score default revisit

- Task: `plan/tasks/144-hnsw-int8-approx-default-revisit.md`
- No code change — measurement packet on `815518d82` (main code state).
- Evidence: `artifacts/manifest.md`.

## Result

`int8_approx` vs the current `exact` default across the full ef sweep:

- **Recall: dips ≤0.42 pp at every point** (10k/50k/100k × ef 40–200) —
  the Task 98 caution does not reproduce beyond noise; the i16 factored
  fallback remains unneeded.
- **Latency: modest, operating-point-dependent win** — −10% at the
  ef64 mid points (50k/100k), neutral at 10k and at ef100 where graph
  traversal (not scoring) dominates.

## Recommendation

Flip `ec_hnsw.turboquant_exact_score_mode` default to `int8_approx`:
it is never worse beyond noise on any measured cell, wins ~10% at the
mid-ef operating points, and unifies both AMs on the same SDOT scorer
path. Flag: the win is small compared to the IVF flip — if the reviewer
prefers, "documented opt-in" is a defensible alternative; the packet
supports either closeout under the task gate ("default flip or
source-grounded negative" — this is a positive, just a modest one).

## Caveats

m5-local only (Graviton deferred, operator 2026-07-03); recall samples
64/48/32 queries.
