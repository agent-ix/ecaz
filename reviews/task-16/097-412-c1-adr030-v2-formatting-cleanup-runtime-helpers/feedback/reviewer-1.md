## Feedback: Formatting Cleanup Runtime Helpers

Read the packet and the three named files.

### What's right

- **Correctly scoped as hygiene, not behavior.** Packet body
  explicitly disclaims semantic or API changes, validation is
  clippy-only (no test surface to report), and the three-file
  scope matches what `rustfmt` would spill after recent runtime
  work.
- **Right response to local diff ambiguity.** The alternative
  ("fold into the next runtime packet") would have made that
  runtime packet's diff noisier. Packaging formatting-only
  cleanup separately is the shape that keeps review small and
  reviewable.
- **No test-surface boilerplate bloat.** Clippy-only validation
  is honest for a formatting packet. Running `cargo pgrx test
  pg17` would have added nothing and is correctly omitted here.

### Concerns

1. **The linker-boundary paragraph is still in the packet body.**
   For a pure-formatting packet it is doubly boilerplate — this
   packet *could not* regress anything a pg test would catch. A
   formatting-only packet doesn't need to name the workstation
   linker family at all. Worth trimming in future hygiene packets
   so the boilerplate doesn't dilute genuinely load-bearing
   linker discussion on behavior packets.
2. **No check that `rustfmt` itself is now a no-op.** The
   packet ships *a* formatting state; it does not confirm that
   `cargo fmt --check` passes after the commit. If this packet
   was produced by hand-wrapping lines rather than running
   `cargo fmt`, a future `cargo fmt` could immediately churn
   these files again. One-line validation — `cargo fmt
   --check` clean at the new SHA — would close that.
3. **Merge order with in-flight packets.** If 416/417 touch
   `src/am/vacuum.rs` or `src/am/scan_debug.rs` (which they do),
   this formatting commit creates a small rebase surface. Not
   load-bearing, just worth noting the sequencing so reviewers
   don't see churn in the other packets' diffs.

### Observation

Standard hygiene packet. The one durable improvement would be
running `cargo fmt --check` as part of the validation list on
formatting packets — that locks the claim "this is now the
formatter's output" rather than "this is one possible format."
