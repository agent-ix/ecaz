# Artifact Manifest

Packet: `reviews/task-111h/034-table-owned-storage-rationale`

Task bucket: `reviews/task-111h`

Head SHA before packet commit:
`c3ce05a542e640b5c5a2613ed5351ef7ea4622a0`

Created: `2026-06-20T20:13:16Z`

## Scope

This is a table-owned storage rationale packet. It does not run a new benchmark
suite. It cites current Task 111h code and existing packet-local benchmark
evidence to close the checklist item that requires either implementation of
table-owned compact payloads or packet-local proof of a concrete blocker plus a
replacement.

## Artifacts

- `table-owned-storage-audit.md`: code-path audit, storage blockers, replacement
  decision, and cited benchmark evidence.

## Commands Used For Audit

```sh
rg -n "RerankPlacement|source_diagnostic|rerank_placement|reserved for real table-owned|Table" \
  src/am/ec_ivf src/tests crates/ecaz-cli/src/commands/bench
sed -n '860,950p' src/am/ec_ivf/options.rs
sed -n '2320,2635p' src/am/ec_ivf/scan.rs
sed -n '920,990p' src/am/ec_ivf/build.rs
sed -n '230,270p' src/am/ec_ivf/insert.rs
sed -n '1,220p' crates/ecaz-cli/src/commands/bench/sidecar_rerank.rs
sed -n '520,900p' crates/ecaz-cli/src/commands/bench/sidecar_rerank.rs
sed -n '1,220p' benchmarks/task51-local-ivf-sidecar-real-io/manifest.md
sed -n '1,220p' reviews/task-111h/029-cross-scale-matched-recall-v7/artifacts/manifest.md
```

## Cited Existing Evidence

- `reviews/task-111h/001-placement-semantics/`: reserves `table`, introduces
  `source`, and renames query-time conversion to `source_diagnostic`.
- `reviews/task-111h/013-rerank-placement-wording/`: makes public reloption
  wording say `table` is reserved.
- `reviews/task-111h/029-cross-scale-matched-recall-v7/`: source/f32 is the
  warm-cache matched-recall reference at 50k, 100k, and 1M.
- `reviews/task-51/016-ivf-sidecar-real-io/` and
  `benchmarks/task51-local-ivf-sidecar-real-io/`: separate companion-table
  sidecar read modes and their measured local costs.

## Key Result Lines

- Current `rerank_placement = 'table'` errors during option resolution and is not
  a product read path.
- Companion-table random-id lookup measured about `16.654 ms` to `18.293 ms`
  p50 sidecar I/O for 50 candidates on the 50k local smoke fixture.
- Companion-table TID-sorted lookup measured about `0.885 ms` to `1.403 ms` p50
  sidecar I/O on the same fixture, but the benchmark manifest states it is a
  microbenchmark with static-corpus and local-cache assumptions.
- Source/f32 remains the warm-cache local reference: at recall target `0.95`,
  packet 029 reports `3.49 ms` p50 at 50k, `6.23 ms` p50 at 100k, and
  `12.2 ms` p50 at 1M.

## Validation

No tests or benchmarks were run. The packet is a code/benchmark-evidence audit.
