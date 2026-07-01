# Task 123 Review Request: Post-A/B Closeout

Please review this synthesis-only closeout request for the reopened Task 123
multi-instance core-algorithm scope.

This packet adds no new run. It asks whether the evidence in packets 017, 018,
and 019 is now sufficient to close Task 123 again as a no-promote / re-scope
result.

Summary:

- Packet 017 is now accepted as the communications datapoint: payload bytes are
  large for `id,source` versus `id`, but bytes shipped are not the dominant
  local latency driver in that relative comparison.
- Packet 018 fixed the dedupe-mode guard so the prune threshold can engage under
  b2/b4 bounded dedupe; reviewer marked the code LGTM and requested explicit
  b2/b4 prune on/off recall + latency A/B.
- Packet 019 provides that A/B:
  - n1024/b2/nprobe64: recall 1.0000 across all variants; prune on/off flat.
  - n128/b4/nprobe96: recall 1.0000 across all variants; prune on/off flat.
  - n1024 p50 is about 0.73s to 0.78s; n128 p50 remains about 5.1s to 5.2s.
  - Per-worker heap payload bytes are 24,632,000 for `id,source` and 16,000 for
    `id`; remote heap dispatch is healthy with no failed dispatches or degraded
    skips.

Requested decision:

1. Confirm packet 019 satisfies the packet 018 required b2/b4 engaged prune A/B.
2. Confirm the reopened Task 123 multi-instance core-algorithm mandate can close
   without a promotion claim.
3. Confirm the closeout wording: the prune fix is retained as correctness /
   plumbing needed for a real lever, but the measured representative b2/b4
   matrix does not validate it as a latency win.

I have not flipped task status in this packet. If you sign off, I will do the
status-sync packet separately.
