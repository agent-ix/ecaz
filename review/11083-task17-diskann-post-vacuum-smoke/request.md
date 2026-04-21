# Review Request: local pg18 DiskANN post-vacuum smoke holds recall

Branch: `adr034-diskann-rebased`
Author: coder-2
Target:

- `review/11083-task17-diskann-post-vacuum-smoke/artifacts/load.log`
- `review/11083-task17-diskann-post-vacuum-smoke/artifacts/pre-vacuum-recall.log`
- `review/11083-task17-diskann-post-vacuum-smoke/artifacts/delete.log`
- `review/11083-task17-diskann-post-vacuum-smoke/artifacts/vacuum.log`
- `review/11083-task17-diskann-post-vacuum-smoke/artifacts/vacuum-timing.log`
- `review/11083-task17-diskann-post-vacuum-smoke/artifacts/post-vacuum-recall.log`
- `review/11083-task17-diskann-post-vacuum-smoke/artifacts/manifest.md`

## What this packet is

This is the local pg18 signoff smoke immediately after packet `11082`.

There is no new code in this packet. The goal was to verify that the two
vacuum-runtime fixes now compose into a usable real-corpus path on the slower
local machine:

1. packet `11081` made the vacuum/scan loops interruptible and capped repair
   scan width at `R`
2. packet `11082` removed the redundant exact-rerank stage from the repair
   frontier planner

The task file now says task 17 is in review/signoff territory rather than
missing AM callback work. This packet is exactly that: a clean real-10k
delete/vacuum/re-recall smoke on pg18 using the canonical `ecaz` path.

## Operator outcome

Using a fresh scratch database (`diskann_vacuum_smoke_c`) and the same real
10k fixture / reloptions as the earlier recovery packets:

- clean pre-vacuum baseline:

```text
│ 128       ┆ 0.9310   ┆ 0.9965 ┆ 82.34 ms    │
```

- deterministic 10% delete:

```text
deleted_rows=1000
```

- `VACUUM (ANALYZE)` completion on the same slower local machine:

```text
VACUUM
Time: 305180.393 ms (05:05.180)
```

- post-vacuum recall on the 9000-row remaining corpus:

```text
│ 128       ┆ 0.9285   ┆ 0.9966 ┆ 81.17 ms    │
```

So the local post-vacuum smoke now completes end-to-end and preserves DiskANN
quality after the delete/vacuum cycle. Recall@10 only moved by `0.0025` on the
same `list_size=128` operating point, while mean query time stayed flat on this
box.

## Why this packet

- It is DiskANN signoff work, not tool work.
- It uses the canonical `ecaz` operator path and the same real fixture as the
  earlier task-17 recovery packets.
- It closes the exact local smoke that `11081` had unblocked but not yet
  completed.
- It respects the machine split: this is a slower-box correctness/smoke check,
  not the final performance bench for the faster machine.

## Validation context

No code changed after `11082`; this packet measures head `f3f5cb0`.

The preceding code checkpoint already passed on `pg18`:

- `cargo test -p ecaz-cli`
- `cargo test`
- `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`

## Follow-ups intentionally not in this packet

- Final performance / tuning claims. Those belong on the faster benchmark
  machine.
- Any more CLI work. The canonical `ecaz` path was sufficient for this smoke.
- Further vacuum optimization beyond the now-completing local real-10k path.
