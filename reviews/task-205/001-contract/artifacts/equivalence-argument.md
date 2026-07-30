# Algorithm 1 pushdown equivalence argument

The implementation stores beam distances as `-IP`. For a live coordinator beam,
the owner threshold is the IP floor corresponding to the worst retained beam
member. An owner drops only neighbors whose code score is worse than that floor;
threshold ties survive and are ordered by `(code_distance, vec_id)`.

The coordinator retains a real candidate heap of size `L`, with
`L >= max(BW, k)`, and sends `t = peek_worst(H_C)` once the bounded heap is
full. It sends the same `l = L` on every round. Each owner applies the threshold
and then truncates once across the merged candidates for the whole request,
not once per requested vec_id. The current BW=4/H=100 gate uses `L=32`, chosen
to make the threshold observable against degree 32 while satisfying the paper's
lower bound for k=10.

The equivalence boundary is explicit: a stale threshold, an L smaller than
`max(BW,k)`, or truncation that is not performed over the merged owner response
could remove a candidate that the bounded coordinator would otherwise retain.
The `None` path remains available for the identity test/reference path. The
cross-scale storage growth row is emitted as measurement only; NFR-021's
fixed-roster interpretation is pending owner resolution.

Evidence for ties, tombstones with `exact_dist = NULL`, mixed owner-like
frontiers, and ordered result identity is in packet
`reviews/task-205/002-implementation/artifacts/pg18-focused.log`.
