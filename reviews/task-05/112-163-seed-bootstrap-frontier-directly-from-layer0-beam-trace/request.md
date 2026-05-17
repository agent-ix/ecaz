# Request: Seed Bootstrap Frontier Directly From Layer-0 Beam Trace

Commit: `5001844`

Summary:
- make bootstrap entry seeding use the graph-owned layer-0 beam runner with the full bootstrap width instead of a single expansion
- seed the visible bootstrap frontier directly from the resulting beam trace
- keep only the seeded entry candidate marked expanded so later discovered candidates remain eligible for refill-on-consume

Please review:
- whether seeding the widened bootstrap frontier directly from the beam trace is the right next A2 runtime handoff
- whether preserving only the entry candidate in expanded-source state is the right compatibility contract for current refill semantics
- whether the new trace-seeding regression coverage is sufficient for this transition
