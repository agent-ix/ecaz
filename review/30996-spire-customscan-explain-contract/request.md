# Review Request: SPIRE CustomScan Explain Contract

- Code commit: `79004738` (`Add SPIRE CustomScan explain contract`)
- Task: Task 30 Phase 12b.3, `ExplainCustomScan` contract
- Scope: narrow behavior-observability slice; no remote dispatch semantic changes

## Summary

This checkpoint wires `EcSpireDistributedScan`'s `ExplainCustomScan`
exec-method callback and extends the existing loopback remote tuple-payload
pg_test fixture with `EXPLAIN (FORMAT JSON, ANALYZE, COSTS OFF)` assertions.

The JSON shape now includes stable scalar properties on the Custom Scan node:

- `node = "EcSpireDistributedScan"`
- `remote_fanout = 1` in the loopback fixture
- `tuple_transport_status = "ready"`
- `nprobe = 2`
- `rerank_width = 0`

## Implementation Notes

- Added `src/am/ec_spire/custom_scan/explain.rs` and included it from
  `custom_scan/mod.rs`.
- `CUSTOM_EXEC_METHODS.ExplainCustomScan` now points at
  `ec_spire_explain_custom_scan`.
- The callback reads the plan-private SPIRE index OID, reuses the existing
  CustomScan eligibility helper for `remote_fanout`, and resolves `nprobe` /
  `rerank_width` from relation/session scan options. It intentionally avoids
  the broad index options snapshot because that diagnostics path can evaluate
  tuple-delivery scannability during EXPLAIN.
- The fixture assertion is attached to
  `test_ec_spire_customscan_returns_loopback_remote_tuple_payload` so it
  exercises the real descriptor, remote fanout, typed tuple payload, and
  CustomScan execution path already used by this packet family.

## Validation

- Passing PG18 focused fixture:
  `review/30996-spire-customscan-explain-contract/artifacts/cargo-test-customscan-explain-contract-pass.log`
- Formatting check:
  `review/30996-spire-customscan-explain-contract/artifacts/cargo-fmt-check.log`

See `artifacts/manifest.md` for commands, timestamps, key lines, and the
diagnostic failed iterations that shaped the final implementation.

## Reviewer Focus

1. Confirm the emitted JSON properties are the right minimal stable contract
   for Phase 12b.3 without overclaiming remote runtime diagnostics.
2. Confirm using `EXPLAIN (FORMAT JSON, ANALYZE, COSTS OFF)` is the right
   fixture form for this exec-method callback.
3. Confirm resolving `nprobe` from configured relation/session scan options is
   acceptable for EXPLAIN, rather than invoking the broader options snapshot.
