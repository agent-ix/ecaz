# Artifact Manifest

- Task: `111a`
- Packet: `reviews/task-111a/008-closeout-decision`
- Head SHA: `432d635dc`
- Timestamp: `2026-06-17T09:45:17-07:00`
- Runner: focused cargo test plus prior `ecaz bench suite` packets

## Code Under Review

- `432d635dc` - expose RaBitQ bits 4/8 arithmetic-batch IVF flush-width
  histograms as width-only block-counter rows.

## Benchmark Evidence Sources

This closeout packet does not rerun the full benchmark matrix. It cites the
accepted task-local benchmark packets:

- `reviews/task-111a/004-all-dense-options-benchmark`
  - TQ and rb1.
  - All six surfaces: row, original dense, dense+coalescing, dense+typed,
    page-spanning packed dense, page-spanning packed dense+typed.
- `reviews/task-111a/007-rabitq-bitwidth-sweep`
  - rb2/rb4/rb8.
  - Same six surfaces, 50k and 100k fixtures.
  - Structured source of record: `artifacts/suite/results.jsonl`.
  - Selected page-spanning EXPLAIN logs committed for copy/page/group counters.

## Artifacts

- `artifacts/closeout.md`
  - AC#6 closeout decision: adopt A + typed views, abandon current B, keep
    dense gated, defer any default promotion to a future 1M/AWS lane.
- `artifacts/focused-test.log`
  - Command:
    `cargo test -q rabitq_bits4_and_bits8_batch_dispatch_use_arithmetic_estimator_with_width_probe --lib`
  - Result:
    `1 passed; 0 failed; 0 ignored; 0 measured; 2116 filtered out`

## Notes

The per-step latency/storage logs from packet 007 remain intentionally
uncommitted. Packet 007's committed `results.jsonl`, `summary.md`, and selected
EXPLAIN logs are the durable evidence source, as allowed by the packet manifest
and confirmed by reviewer feedback.
