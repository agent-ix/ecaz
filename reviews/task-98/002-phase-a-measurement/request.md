# Task 98 Packet 002: Phase A Measurement — Complete, with Scope-Down Call

Phase A is complete. Three findings landed as commits along the way (all
under review here): the binary-prefilter shadowing of exact modes, the V3
hot/cold inline-payload root cause that had silently killed every
exact-mode batching arm including Task 87's FullLut (`a1122aac8`), and the
stale-extension-catalog display failure with its CLI warning fix
(`eb7183a65`).

With those fixed, the full 24-cell matrix is measured (manifest has the
tables):

- **Width distribution (criterion 4): ≥32-wide flushes are 0.025–0.081%**
  across real10k/50k/100k — mean width ~2.5–3. Per the task's stop
  condition (<20%), **Phase C SVE cloud measurement is skipped**; the
  partial-width dispatch is where SIMD pays on this surface.
- **Recall byte-equal** kernel-on vs kernel-off at all six (mode × corpus)
  cells.
- **int8_approx32**: full `isa=neon` coverage, integer-exact, kernel-on
  p50 faster at 100k (5.76 vs 6.92 ms).
- **tiled_lut32**: scalar reference this phase; no consistent end-to-end
  regression; any tiled SIMD slice would take the partial-width form and
  is deferred to the Intel/AVX2 lane question.

## Review request

Please review the three findings, the scope-down call, and the cell
evidence. With this packet, Task 98's local scope (kernels, routing,
instrumentation, Phase A decision data) is complete; remaining items are
the AVX2 lane question (Intel host) and closeout aggregation under
Task 99.
