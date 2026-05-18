# Review Request: Task 29 DiskANN graph diagnostics

Branch: `task29-diskann-initial-tuning`
Author: coder1

## What Changed

Added a read-only DiskANN graph diagnostics surface and CLI wrapper:

- `ec_diskann_index_graph_summary(index_oid oid)` returns metric/value rows for
  persisted graph shape.
- `ecaz bench diskann-graph --prefix ... --log-output ...` calls that function
  through the normal `ecaz-cli` connection path.

This keeps Task 29 graph-quality probes out of bare `psql` and avoids ad-hoc SQL
for the next optimization decisions.

## Validation

Code validation before the code commits:

- `cargo test -p ecaz-cli graph::tests`
- `cargo test --lib am::ec_diskann::diagnostics`
- `cargo check --all-targets --no-default-features --features pg18`
- `cargo pgrx test pg18 pg_test_ec_diskann_`
- `cargo pgrx install --pg-config /home/peter/.pgrx/18.3/pgrx-install/bin/pg_config --no-default-features --features pg18`
- `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
- `git diff --check`

The small follow-up CLI cast fix was validated with:

- `cargo test -p ecaz-cli graph::tests`
- `cargo check -p ecaz-cli`
- `git diff --check`

## Measurement Surface

Database: `task29_diskann_baseline`

Compared two already-loaded isolated real-10k DiskANN indexes:

- `task29_diskann_real10k_idx`: baseline from packet `676`
- `task29_diskann_prior_real10k_idx`: prior-neighbor fix from packet `678`

The benchmark database did not yet have the new SQL wrapper because the
extension version is unchanged, so this packet applies the one wrapper function
through `ecaz-cli dev sql` using packet-local SQL. The corpus and indexes were
not dropped or rebuilt.

## Results

Baseline graph (`artifacts/graph-diskann-baseline.log`):

- `reachable_live_node_count = 9999 / 10000`
- no dead, invalid, self, duplicate, or unresolvable neighbor refs
- out degree avg `22.7822`, p50 `22`, p95 `32`, max `32`
- in degree avg `22.7822`, p50 `21`, p95 `41`, p99 `59`, max `3800`

Prior-neighbor graph (`artifacts/graph-diskann-prior.log`):

- `reachable_live_node_count = 10000 / 10000`
- no dead, invalid, self, duplicate, or unresolvable neighbor refs
- out degree avg `24.5035`, p50 `25`, p95 `32`, max `32`
- in degree avg `24.5035`, p50 `22`, p95 `43`, p99 `61`, max `3250`

## Interpretation

The current DiskANN quality gap is not explained by disconnected persistence or
obvious corrupt neighbor slots. The graph is clean and effectively fully
reachable.

The stronger signal is graph shape: both builds have extreme in-degree hubbing
while most nodes receive around 20-40 incoming links. The prior-neighbor fix
improves reachability and average out degree, but recall still plateaus near
`0.9315`, so the next optimization should target Vamana build candidate
selection/diversification rather than scan `list_size`, `alpha`, or persistence
repair.

## Recommendation

Keep the diagnostics command. For the next code slice, instrument and optimize
build-side graph shape:

- measure candidate pool sizes and selected neighbor diversity during build
- reduce hub concentration in `robust_prune` input selection
- then rebuild real-10k and re-run the same packet `676` recall/latency grid

Landing blocker remains: DiskANN is structurally correct, but its graph quality
does not yet match the HNSW reference row from packet `676`.
