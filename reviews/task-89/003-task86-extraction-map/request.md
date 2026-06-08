# Task 89 / Packet 003: Task 86 TQ+ Extraction Map

## Summary

This packet maps the reverted Task 86 TQ+ code onto the ADR-076 production
shape. It identifies the shared math that should be reused and the old IVF
storage/API pieces that should not be re-landed verbatim.

No code porting is included. This preserves the Task 89 Phase 1 rule that the
format ADR should be reviewer-approved before Phase 2 implementation starts.

## Artifact

- `artifacts/task86-tqplus-extraction-map.md`

## Validation

Documentation-only extraction map. No Rust tests were run.

Commands/source material inspected:

- `git show e0ae9fe7d -- src/quant/prod.rs`
- `git show c7e85e8ac -- src/quant/prod.rs`
- `git show e0ae9fe7d -- src/am/ec_ivf/{options,page,quantizer,build,insert,scan}.rs`
- `git show c7e85e8ac -- src/am/ec_ivf/{quantizer,build}.rs`
- `git show 55e492899 -- src/am/ec_ivf/page.rs`

## Reviewer Focus

Please confirm the proposed first post-ADR code slice:

1. Reintroduce shared TQ+ math in `src/quant/prod.rs`.
2. Add shared unit coverage.
3. Avoid AM reloption/page-layout changes until the shared math review passes.
