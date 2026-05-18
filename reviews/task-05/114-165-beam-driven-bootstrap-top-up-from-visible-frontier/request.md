# Request: Beam-Driven Bootstrap Top-Up From Visible Frontier

Commit: `e0e1620`

Summary:
- replace the remaining production bootstrap top-up loop with a graph-owned layer-0 beam expansion seeded from the still-visible frontier
- keep consumed-source refill as the first step, then let graph-owned beam expansion discover additional candidates from remaining visible seeds
- mark expanded sources from the resulting beam trace so the runtime no longer relies on the old local scheduler loop for production top-up

Please review:
- whether using visible frontier candidates as seeds for post-success top-up is the right next A2 runtime handoff
- whether the current expansion budget of `bootstrap_limit - visible_len` is the right compatibility contract for this step
- whether keeping the old generic top-up helper only for tests is an acceptable temporary split
