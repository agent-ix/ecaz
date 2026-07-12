# Verdict: retain the validated physical epoch cache

Retain the bounded backend-local cache enabled by default.

The 10k/50k/100k A/B is recall-neutral and storage-neutral within one or two
PostgreSQL pages, while warm physical p50 improves by 99.16% to 99.54%. Mean and
p95 improve by more than 94% at every scale despite including one cold validation
per benchmark child.

The evidence supports the implementation's narrow scope:

- exact key: index OID, logical index UUID, build ID, epoch fingerprint;
- cached value: already validated descriptor digest, descriptor, and bounded head graph;
- capacity: two entries with backend-local LRU eviction; and
- exclusions: conninfo, relation handles, pointer state, and scan tokens are not cached.

The cache does not alter generation contents or query ranking. Exact physical
recall is unchanged at all three scales, storage differences are page-layout
noise, topology is complete, and both remote owners are proven through the
distributed CustomScan in every arm.

This packet closes the performance-evidence question for the cache slice only.
It does not close Task 179: remote transport timeouts/interruptibility, real
three-instance partial-window fault injection, head-cap sensitivity, and outside
review of Task 172's broader benchmark packet remain open.
