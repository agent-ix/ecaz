# Task 111a Closeout

## Accepted Benchmark Coverage

The accepted 111a evidence is split across two benchmark packets:

- Packet 004 covers TurboQuant and rb1.
- Packet 007 covers rb2, rb4, and rb8.

Together they cover TQ plus RaBitQ `{1,2,4,8}` over the requested surfaces:

- row postings
- original dense postings
- original dense with coalescing
- original dense with typed views
- page-spanning packed dense
- page-spanning packed dense with typed views

Packet 007 reviewer feedback independently verified recall parity, the latency
and storage tables, and the page-spanning copy/page counters. The reviewer
concluded that the matrix is complete and that B is dominated.

## AC#6 Recommendation

Recommendation: adopt Approach A plus typed views, abandon the current
page-spanning packed dense physical format, and keep dense gated.

Details:

- Approach A, scan-side coalescing, is the right fix for TurboQuant where the
  original dense layout fragmented scorer batches.
- The typed dense view stays. It wins rb4 at 100k and remains competitive
  elsewhere without changing recall.
- RaBitQ should prefer the simple one-page dense/typed layout. The RaBitQ
  kernels are cheap enough that smaller footprint and fewer page reads dominate
  the explicit coalescing benefit in most rb2/rb4/rb8 cells.
- The current page-spanning packed format should not ship as the durable format.
  It functionally exercises option 3, but the measured copy cost and segment
  fragmentation dominate: packet 007 reports 16.6 MB copied for rb2, 32.8 MB
  for rb4, and 65.2 MB for rb8 per representative 100k scan.
- Do not promote `dense_posting_blocks` to default from these local 50k/100k
  runs. A 1M/AWS run is still required before any default promotion decision,
  but no default move is recommended by this closeout.

## Task 42 / Tag Follow-Up

Before promotion, reconcile the final on-disk tag set for Task 42:

- Keep the simple dense tag.
- Keep the aligned/typed dense tag if typed views remain durable.
- Retire or quarantine the packed page-spanning tags if Approach B stays
  abandoned.

## Future Spanning Format

This does not rule out a future spanning design. If page-spanning dense is
revisited, the next design should avoid the current assembly-copy cost. The
likely direction is a scorer that can consume segmented payload fragments
directly, or a continuation layout that materially reduces page reads and avoids
rebuilding contiguous payload slabs during scan.
