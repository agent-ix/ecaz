## Feedback: TurboQuant quantized default — ACCEPTED

Verified against:

- commit `89398ec` (`Default source-backed turboquant rerank to quantized`)
- `src/am/scan.rs` `default_grouped_rerank_mode(...)` now keyed on
  `StorageFormat::PqFastScan` rather than any index with
  `build_source_column`
- new `PqFastScanRerankModeResolution::
  DefaultQuantizedTurboQuantStorage` resolution name

### What's right

- **Policy-only, not implementation.** The rerank machinery from
  `424` is untouched. This packet only flips the default for
  turboquant indexes. That is exactly the right size for a "change
  your mind about a default" diff.
- **`heap_f32` remains an explicit opt-in.** Keeping the env
  override means a measurement packet can still force heap-f32 on
  turboquant without rebuilding, which is what `426`/`429`/`430`
  needed.
- **Resolution reason enum named, not inferred.** Adding
  `DefaultQuantizedTurboQuantStorage` means `pg_settings`-style
  debug output now tells you *why* turboquant picked quantized,
  instead of this decision being silent. Good debuggability.
- **Unit coverage splits the three cases cleanly.** Source-backed
  pq_fastscan → heap_f32, source-backed turboquant → quantized,
  source-less pq_fastscan → quantized. All three needed to be
  pinned separately; all three are now pinned.

### Concerns

1. **No explanatory comment on the policy itself.** The `scan.rs`
   call site reads "PqFastScan + source → heap_f32, else
   quantized." A future reader who hasn't lived through the `424 →
   425` arc will not know that source-backed turboquant is
   intentionally excluded because measurement showed heap-f32 was
   slower there. Worth a one-line `// see packet 426 measurement —`
   referencing the non-obvious asymmetry, or a comment on the
   resolution enum variant.
2. **Packet does not ship with a before-after latency row.** The
   justification relies on packet `424`'s `5.220ms` vs `3.005ms`
   numbers. Those exist on disk but aren't cross-linked here. Fine
   for a policy-only diff but worth naming the artifacts once.
3. **`pq_fastscan` source-less default is unchanged here but
   depends on the same branch.** A later refactor that flattens the
   two conditions could accidentally flip source-less pq_fastscan
   without anyone noticing. The new unit tests do catch this, but
   only because all three cases are pinned — worth leaving that
   test coverage comment visible.

### Call

Accepted. This is a defensible default flip backed by packet `424`'s
measurement and with matching unit coverage. Pair with packet `426`
for the on-the-wire proof.
