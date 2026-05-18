# Review Request: SPIRE CLI Benchmark Profile

- Code commit: `b290560c` (`Add SPIRE CLI benchmark profile`)
- Branch: `task30-spire-partition-object-spec`
- Task: Task 30 SPIRE IVF foundation
- Agent: coder1

## Summary

This checkpoint registers `ec_spire` in the `ecaz-cli` benchmark profile
registry so the existing corpus loader, recall harness, and latency harness can
exercise the SPIRE access method directly.

The profile uses:

- access method: `ec_spire`
- opclass: `ecvector_spire_ip_ops`
- scan GUC: `ec_spire.nprobe`
- raw `real[]` KNN query binding, matching the SPIRE opclass operator shape
- SPIRE reloption help keys, including Phase 4 `local_store_count` and
  `local_store_tablespaces`

While touching the registry, this also fills the previously empty `ec_ivf`
default nprobe sweep so the existing "every registered profile has a default
sweep" invariant passes.

## Review Focus

1. Confirm that the profile matches the SQL bootstrap SPIRE opclass contract.
2. Confirm that the known reloption list is accurate enough for loader warnings.
3. Confirm that adding an `ec_ivf` default sweep is acceptable registry drift
   cleanup rather than an unrelated benchmark behavior change.

## Validation

- `cargo test -p ecaz-cli profiles`
- `cargo test -p ecaz-cli psql::tests::spire_index_sql_uses_spire_access_method_and_opclass`
- `cargo test -p ecaz-cli build_knn_sql_uses_raw_real_query_for_spire`
- `cargo fmt --check`
- `git diff --check`

## Notes

This packet is intentionally separate from the measured SPIRE recall/latency
gate. It only adds the reusable CLI surface needed for that measurement packet.
