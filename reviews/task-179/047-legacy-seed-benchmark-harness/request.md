# Review request: isolated legacy seed benchmark control

## Scope

Please review implementation commit `2bf203e4c` as the measurement harness for
the remaining packet 030 P2-1 closeout gate: persisted-head seeding versus the
removed owner-wide O(N) seed scan.

Comparing against the historical pre-head checkout would also remove later
multicluster correctness, recovery, transport, projection, and fanout changes.
This checkpoint instead adds a default-off
`distann-legacy-seed-benchmark` Cargo feature that changes only physical seed
acquisition on otherwise-current code:

- the normal build still creates and persists the same bounded head sample;
- normal production builds continue to load/search that persisted head;
- the benchmark feature skips head loading at scan open and restores a full
  graph-relation scan, decode, and approximate scoring pass on every owner;
- remote owner scans use the current bounded, pooled, concurrent transport;
- merged seeds retain the same global distance ordering and seed limit before
  entering the unchanged distributed graph search; and
- `ec_distann_physical_seed_strategy()` plus structured fixture output records
  `owner_scan`, `persisted_head`, or the same-data `single_index` control in
  durable suite results.

The owner-scan SQL endpoint is compiled only with the benchmark feature. The
feature is empty/default-off in `Cargo.toml` and is explicitly documented as
non-production.

## Validation

See `artifacts/manifest.md`. The exact committed tree passes the existing live
PG18 three-owner build/publish/read fixture with the benchmark feature enabled,
including its assertion that the compiled strategy is `owner_scan`. The suite
parser regression passes with the new structured field. Strict PG18 clippy
passes both with normal production features and with the benchmark feature.

This packet validates the control harness only. It makes no performance claim;
the following packet will run the canonical 10k/50k/100k A/B recall, warmed
latency, and storage matrix with both arms built from this exact code commit.

## Requested decision

Please confirm that the feature provides an isolated and conservative control
for the removed O(N) owner seed acquisition, without changing the production
path or the generation built for either A/B arm.
