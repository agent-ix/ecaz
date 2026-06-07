# Task 85 Review Request: AWS 1M Per-Leaf Block Cap Suite

## Summary

Packet 004 rejected `leaf_block_rows=8`: it halved candidate rows at the
retained recall point but worsened p50/p95 because leaf object read and
summary/row scoring time increased. This packet tests a query-only alternative
on the retained block16 index: use a per-leaf block cap instead of a global
block cap.

The headline row is `perleaf12`. With `nprobe=96`, it targets the same nominal
block budget as the retained global cap:

- retained global cap: `1152` blocks per query;
- per-leaf cap: `96 routes * 12 blocks = 1152` blocks per query;
- block16 rows: nominally `1152 * 16 = 18,432` candidates per query.

This is not accepted unless it preserves `recall@10 >= 0.9832` and beats the
same-suite global1152 warm control on latency.

## Suite

`reviews/task-85/005-aws-1m-per-leaf-block-cap/suite-aws-1m-per-leaf-block-cap-q500.json`

Rows:

- same-suite retained global1152 first control;
- perleaf8, perleaf10, perleaf12, perleaf14 sensitivity rows;
- same-suite retained global1152 warm repeat control.

## Acceptance Bar

A per-leaf row can only claim a Task85 latency improvement if it has:

- `recall@10 >= 0.9832`;
- p50/p95/p99 better than the same-suite global1152 warm control;
- candidate sum at or below the retained `9,213,846` q500 surface for the
  accepted row;
- no storage/build regression, since this is query-only on the retained index.

## Validation

- `ecaz bench suite audit`: passed for 7 steps.
- AWS run: pending.

## Requested Review

Please review whether this is the right next query-only latency mechanism after
the block8 rejection. The risk is recall loss from replacing global block
allocation with equal per-leaf allocation; the suite is designed to measure that
directly.
