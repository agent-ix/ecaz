# Review Request: SPIRE Larger Multi-Store Fixture

- Code commit: `81a845c6` (`Cover larger SPIRE multistore fixture`)
- Branch: `task30-spire-partition-object-spec`
- Task: Task 30 SPIRE IVF foundation, Phase 4 local placement coverage
- Agent: coder1

## Summary

This checkpoint adds the Phase 4-specific larger PG18 multi-store regression
fixture requested in the final landing review, while leaving repo-wide
hardening tasks for later packets.

The new test `test_ec_spire_multistore_large_fixture_routes_all_stores` builds
a relation-backed SPIRE index with:

- 256 deterministic rows;
- 384-dimensional `ecvector` embeddings;
- 32 lists with `nprobe = 8` and `rerank_width = 25`;
- 4 local stores using repeated `pg_default` tablespaces for same-device
  baseline coverage.

The test asserts:

- active diagnostics report 4 local stores;
- object and placement snapshots span all 4 local stores;
- the routing object has 32 children;
- routing object bytes exceed a single page, exercising the routing-object
  chain path under a multi-store build;
- relation storage diagnostics see multiple relation blocks;
- ordered scan still returns a full top-10 result.

The task tracker and status row now record the larger four-store fixture and
move T30 to 93%.

## Review Focus

1. Confirm the fixture is scoped correctly to Phase 4: relation-backed
   multi-store build, placement distribution, routing-chain size, diagnostics,
   and scan sanity.
2. Check that repeated `pg_default` tablespaces are appropriate here; this is
   coverage for the same-device baseline path, not another hardware benchmark.
3. Verify the deterministic SQL data generation is stable enough for PG18 test
   use without adding external fixtures.
4. Confirm the tracker/status wording does not imply production multi-NVMe
   performance claims.

## Validation

- `cargo fmt --check`
- `git diff --check`
- `cargo pgrx test pg18 test_ec_spire_multistore_large_fixture_routes_all_stores`

PG17 was not run; this is a PG18 Phase 4 relation-backed local multi-store
coverage slice.

## Notes

The broader `30509` checklist items such as `include!` to `mod`, typed errors,
rustdoc, unsafe comments, codec property tests, and behavioral autovacuum
timing tests remain intentionally out of this packet. Those are valuable, but
they are broader subsystem hardening rather than Phase 4 local-placement
closure.
