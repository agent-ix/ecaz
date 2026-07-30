# Algorithm 1 pushdown equivalence argument

The implementation stores beam distances as `-IP`. For a live coordinator beam,
the owner threshold is the IP floor corresponding to the worst retained beam
member. An owner drops only neighbors whose code score is worse than that floor;
threshold ties survive and are ordered by `(code_distance, vec_id)`.

The threshold is sent only when the live beam already contains at least the
remaining expansion budget (`remaining rounds × BW`). Before that point, the
threshold is `NULL`, so a candidate that could still be needed to fill the
bounded beam cannot be removed. The owner limit is that remaining budget, with a
minimum of one for the final response; it bounds each response to candidates
that can be consumed by the remaining rounds. The coordinator retains its
existing visited-set, exact-distance, tombstone, early-exit, and final ordering
rules.

The equivalence boundary is explicit: a stale threshold from an earlier round,
or an `l` smaller than the number of candidates the remaining beam can consume,
could remove a candidate that the coordinator would otherwise retain. The
production derivation avoids both conditions; the `None` path remains available
for the identity test/reference path.

Evidence for ties, tombstones with `exact_dist = NULL`, mixed owner-like
frontiers, and ordered result identity is in packet
`reviews/task-205/002-implementation/artifacts/pg18-focused.log`.
