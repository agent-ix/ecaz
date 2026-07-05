# Review Request: AM Mod Cost Vacuum Split

Scope:
- `src/am/mod.rs`
- `src/am/cost.rs`
- `src/am/vacuum.rs`

What changed:
- Extracted the planner cost callback from `src/am/mod.rs` into `src/am/cost.rs`.
- Extracted the vacuum no-op callbacks from `src/am/mod.rs` into `src/am/vacuum.rs`.
- Left behavior unchanged and kept the existing helper/test surface intact so the split is mechanical.

Review focus:
- Whether the extracted callback modules preserve the existing SQL-visible and planner-visible behavior exactly
- Whether the helper visibility boundary between `mod.rs` and the new modules is narrow and appropriate
- Whether this split reduces scan-work merge risk without introducing unnecessary module churn

Questions to answer:
- Do the extracted cost and vacuum callbacks remain behavior-identical to the prior `mod.rs` implementation?
- Are the remaining shared helpers in `mod.rs` exposed at the right visibility for these modules?
- Is this the right first `am/mod.rs` split before moving on to larger build/scan module extraction?
