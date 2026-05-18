# Review Request: AM Options Module Split

Scope:
- `src/am/mod.rs`
- `src/am/options.rs`

What changed:
- Extracted relation-option definitions, reloption parsing, and the `amoptions` callback from `src/am/mod.rs` into `src/am/options.rs`.
- Kept the existing build and insert callers using the same parsed `TqHnswOptions` shape.
- Left behavior unchanged so this remains a mechanical review-surface cleanup before scan work.

Review focus:
- Whether the extracted reloptions module preserves current reloption parsing and validation behavior exactly
- Whether the `TqHnswOptions` visibility boundary is appropriate for the remaining build/test code in `mod.rs`
- Whether this split is a sensible staging step before extracting larger build or scan modules

Questions to answer:
- Does the new options module preserve all current reloption defaults, bounds, and string parsing behavior?
- Is the shared `TqHnswOptions` type exposed narrowly enough for the remaining callers and tests?
- Does this split reduce `am/mod.rs` review risk without obscuring the current AM registration flow?
