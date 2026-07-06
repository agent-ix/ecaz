# Task 145 Packet 010: Bound-Prune Engagement Counter

## Request

Please review code commit `6eb1e46ab2f88a3aadce316dd0eac6994a4d999a`.

This is a remediation checkpoint for the faulty Task 145 packet 008 bound-prune A/B. Packet 008 did not prove that the intended mechanism fired, so its bound-prune conclusion is rejected as null/faulty evidence. A correct conclusion requires a direct engagement counter plus a new A/B that demonstrates the mechanism actually ran.

## What Changed

- Added a dedicated `pre_materialization_pruned_candidate_row_count` counter.
- Incremented it only from the true pre-materialization prune branches, before row materialization.
- Preserved the existing `truncated_candidate_row_count` as an inclusive compatibility counter.
- Surfaced the new counter through:
  - scan diagnostics
  - selected-leaf scan profiles
  - `ec_spire_index_scan_leaf_candidate_snapshot`
  - `ec_spire_remote_search_production_scan_profile`
  - `ec_spire_remote_search_coordinator_local_scan_profile`
  - remote libpq profile decoding
  - `ecaz bench spire-pipeline` production scan-profile aggregation as `pre_materialization_pruned_sum`

## Validation

See `artifacts/manifest.md`.

All focused validation passed:

- selected-leaf profile counter test
- placement diagnostics counter test
- CLI production scan-profile rendering test
- CLI SQL contract test

## Non-Claims

This packet does not close AC2 bound-prune and does not make any latency/recall conclusion. It only installs the missing observability needed to prevent another inert A/B from being treated as evidence.

The next required step is a real `ecaz bench suite` bound-prune re-run where the packet must show `pre_materialization_pruned_sum` is `0` for off and nonzero for on before any performance interpretation is valid.
