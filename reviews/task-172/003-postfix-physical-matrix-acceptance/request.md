# Review request: post-fix physical matrix for Task 179 AC-13

## Scope

Please review the immutable Task 179 packet 052 candidate as the missing
post-fix latency arm from Task 172 packet 002's outside-review disposition.
This request is deliberately narrow: decide whether Task 179 acceptance
criterion 13 now has reviewer-accepted 10k/50k/100k physical-versus-single
recall, warmed latency, storage, and topology evidence.

This is not a request to close or promote Task 172 itself. Its required
throughput curve, full distributed telemetry, instrumentation-overhead audit,
and 1m/10m capacity model remain open.

## Why this supersedes the rejected latency arm

Packet 002 measured extension SHA `77e09a511` before five named hot-path fixes,
used only five latency iterations, and mixed a cold first query into an arm
labeled warm. The current immutable measurement is:

- extension source `9387f72b3` (prompt-cancel implementation `a94e5e9be`);
- persisted-head seeding, cached physical epoch/head state, parallel remote
  fanout, direct native graph reads, guarded scan tokens, and prompt transport
  cancellation all present;
- 10 same-connection untimed warmups before each arm;
- 50 measured queries at concurrency 1 per scale; and
- 20 recall queries / 200 recall@10 membership trials per scale.

Packet 052 isolates the final prompt-cancel poll against packet 050's already
post-fix direct-reader baseline and finds no material poll overhead. Packet 050
in turn isolates the direct reader; packet 048 establishes the persisted-head
production baseline versus the removed O(N) owner scan. Every packet is
canonical `ecaz bench suite` evidence.

## Current matrix

| Scale | Physical recall@10 | Single recall@10 | Physical mean / p50 / p95 / p99 ms | Single mean / p50 / p95 / p99 ms | Physical generation bytes |
| --- | ---: | ---: | ---: | ---: | ---: |
| 10k | 1.0000 | 1.0000 | 43.50 / 44.00 / 55.70 / 56.10 | 2.83 / 2.77 / 3.43 / 3.57 | 242,761,728 |
| 50k | 0.9800 | 0.9750 | 54.50 / 54.20 / 67.90 / 72.30 | 3.38 / 3.43 / 3.98 / 4.15 | 1,242,734,592 |
| 100k | 0.9500 | 0.9450 | 49.50 / 46.90 / 67.40 / 75.90 | 3.56 / 3.39 / 4.55 / 4.88 | 2,496,634,880 |

Distributed recall is equal to or above the same-run single control at every
scale. Physical mean latency remains roughly 13.9–16.1x the local single-index
control, a product finding rather than hidden overhead, but it is now measured
on the representative post-fix design with a genuinely warmed sample.

Storage amplification against `rows * 1536 * 4` raw f32 bytes is 3.9512x,
4.0454x, and 4.0635x. As packet 002 already ruled, the measurement is valid but
the 50k/100k points do not support NFR-018 promotion.

All scales prove exact 10k/50k/100k global record coverage, zero non-owned
rows, zero orphans, and two remote owners. At 100k the owner counts are
33,195 / 33,432 / 33,373; maximum deviation from the mean is 0.415%, correcting
packet 002's untraceable 0.296% statement.

The benchmark command's `--top-k 10` is the returned/evaluated recall@10. The
standard profile sets the internal `ec_distann.top_k` search/rerank tuning point
to 32, which is why the detailed recall/latency tables display `top_k=32` while
`recall_trials=200` equals 20 queries times 10 evaluated memberships.

## Requested decision

Please explicitly decide whether the post-fix warmed latency matrix closes the
remaining open latency axis from packet 002 and therefore satisfies Task 179
AC-13 in combination with packet 002's already accepted recall, storage, and
topology axes. Please keep broader Task 172 closure open.
