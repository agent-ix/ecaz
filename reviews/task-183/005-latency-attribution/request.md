---
task: 183
packet: 005-latency-attribution
role: coder
status: open
date: 2026-07-17
head: 8c47c9408
---

# Review request: latency attribution pre-registration

Phase 2 produced no recall candidate and Phase 3 was conditionally skipped.
Phase 4 therefore profiles only the retained Task 182 production policy:
trained cap 4,096, exact scoring, 32 returned seeds, BW4/H100, RaBitQ neighbor
traversal, and exact final ranking on the physical three-owner path.

Before reading a new stage profile, this packet freezes benchmark-only
per-backend aggregate counters with the following non-overlapping or explicitly
nested stages:

1. query/codec preparation;
2. exact dot-product scoring of all persisted landmarks;
3. seed ranking/truncation after scoring;
4. total bounded traversal, with nested local-owner expansion and remote-owner
   expansion/transport counters;
5. remote result-payload materialization;
6. ranked-output construction/merge; and
7. total CustomScan search/materialization setup.

The client-side mean minus mean total CustomScan setup is reported as the
executor/client residual. Traversal control/merge is derived as total traversal
minus local and remote expansion. Nested counters will never be summed as if
they were independent stages.

The counters reset after warmups and snapshot after timed queries on the same
latency-worker backend. They are compiled only with the existing
`distann-head-attribution-benchmark` feature and do not alter production
defaults or persisted data. The CLI will fail closed if the requested snapshot
functions are unavailable.

The initial profile is one fresh 100k physical generation, 50 timed queries
after 10 warmups, driven by a checked-in `ecaz bench suite` config. No latency
optimization is selected yet. After the profile:

- head scoring dominance permits one contiguous/vectorized exact-scoring A/B,
  with scalar equivalence and seed-identity checks;
- seed ranking dominance permits one bounded selection implementation with
  identical ordered seeds;
- a reproduced 10k-only regression without recall gain permits a small-corpus
  bypass; and
- dominance by traversal, remote transport/materialization, or executor
  residual yields no Task 183 latency candidate because those are outside the
  eligible isolated changes.

Any selected change is measured alone against byte-identical index and query
inputs. Only a useful isolated candidate proceeds to the task's required
10k/50k/100k confirmation; otherwise packet 006 records a stop decision.
