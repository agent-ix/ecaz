## Feedback: PqFastScan Pg-Test Surface Rename

Read the renamed test function names, SQL fixtures, and the changed
reloption test input in `src/am/options.rs`.

### What's right

- **Completes the rename sweep that 386 started.** Test function
  names, SQL table/index names, open-index labels, and the last
  `grouped-v2` string in the negative-parse reloption test all move
  to `pq_fastscan`. After this packet, `grouped_v2` should only
  survive in backcompat aliases and wire-tag constants.
- **Pragmatic PostgreSQL 63-byte identifier fix.** Shortening the
  four test names that spilled past the NAMEDATALEN limit is the
  correct response — pgrx rejects them otherwise. Worth noting the
  general pattern: any future rename that lengthens identifiers
  needs this check.
- **Negative-parse test input now uses `legacy_format`** instead
  of the old public name. That keeps the test's intent (reject
  anything other than `turboquant`/`pq_fastscan`) while not
  advertising the removed name as a real thing users might type.

### Concerns

1. **Did anything survive the sweep?** Worth a one-line grep
   report in the packet body: "`rg grouped_v2` returns N lines,
   all in [list of allowed places]." Without that it's hard to
   tell whether the sweep is actually complete.
2. **Wire-tag constants still reference grouped-v2 terminology
   by name.** `INDEX_FORMAT_V2_GROUPED` is the on-disk version
   byte and must not rename, but a reader of the rename packet
   won't see that distinction without a comment. This is the
   same concern flagged on 384.
3. **Linker gap.** Pure rename; minimal risk. But the sheer
   volume of renamed tests (any test that previously exercised
   grouped build/insert/scan/vacuum) means this is the packet
   most likely to harbor a missed test-name reference that only
   pgrx compilation surfaces. Clippy + cargo check do catch those
   at compile time, so risk is low.

### Observation

With 386 + 388 landed, the product/runtime naming mismatch is
essentially gone from the source surface. The remaining `grouped_v2`
leakage is exactly where it should be: wire bytes and compatibility
aliases.
