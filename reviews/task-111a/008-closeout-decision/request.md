# Review Request: Task 111a Closeout Decision

## Scope

This packet closes out Task 111a after the full matrix in packets
`004-all-dense-options-benchmark` and `007-rabitq-bitwidth-sweep` and after the
reviewer feedback on packet 007.

It also includes one narrow code checkpoint, `432d635dc`, which makes RaBitQ
bits 4 and 8 arithmetic-batch IVF scans emit width-only block-counter rows.
Those lanes intentionally keep the faster arithmetic estimator instead of the
block-kernel wrapper, so the new hook records only flush-width histogram samples
and does not attribute kernel/scalar scoring work.

## Decision

Task 111a should close with this recommendation:

- Adopt Approach A, scan-side dense coalescing, for the dense path that needs it
  most: TurboQuant.
- Keep the aligned typed-view dense layout. It is the rb4 winner in packet 007
  and competitive elsewhere.
- For RaBitQ, keep the simple one-page dense/typed layout as the durable winner;
  coalescing is not required for the current RaBitQ kernels.
- Abandon the current page-spanning packed dense format, Approach B, as the
  durable shape. It was implemented and measured, and it is dominated on
  latency/storage/page reads/copy bytes across TQ plus RaBitQ `{1,2,4,8}` at
  equal recall.
- Keep `dense_posting_blocks` gated. Do not promote it to default from this
  local 50k/100k evidence. A 1M/AWS lane remains a prerequisite only if default
  promotion is put back on the table.

## Evidence

- Packet `004-all-dense-options-benchmark`: TQ and rb1 across all six surfaces.
- Packet `007-rabitq-bitwidth-sweep`: rb2/rb4/rb8 across all six surfaces.
- Packet 007 reviewer feedback confirms the matrix is complete and B is
  dominated.
- `artifacts/closeout.md` records the final AC#6 recommendation and remaining
  follow-ups.
- `artifacts/focused-test.log` records the focused unit test for the new
  width-only rb4/rb8 counter hook.

## Validation

```text
cargo test -q rabitq_bits4_and_bits8_batch_dispatch_use_arithmetic_estimator_with_width_probe --lib
```

Result: 1 passed, 0 failed.

`cargo fmt --check` was attempted before this packet and still reports existing
unrelated formatting drift in files outside this slice, so no repo-wide format
pass was applied.
