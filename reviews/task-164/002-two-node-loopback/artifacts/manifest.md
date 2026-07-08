# Task 164 M2 — two-node loopback (TC-040/041 + H×RTT) manifest

## Provenance

- **Branch / head SHA:** `task-164-ec-distann-m2` @ `4d5093545` (transport
  rework: parallel batch + parameterized session setup + typed errors).
- **Build:** release `.so` (`cargo pgrx install --release … pg18`);
  `ecaz_build_profile()` = `release` verified on `ec_distann_m2`.
- **Host:** Intel desktop, PG18 port 28818, socket `/home/peter/.pgrx`.
- **Substrate:** ADR-085 D2 loopback multi-instance, reduced to a single
  committed instance — the "2-node" roster points both entries at this
  instance, and each remote call sets its target `local_node_id` on the
  session, so one instance serves both hash partitions over real
  tokio-postgres round-trips. (pg_test can't do this: its rolled-back txn hides
  data from a second connection, so this runs against the committed
  `ec_distann_m2` DB, not the pg_test harness.)

## Fixtures

- **Real 10k / dim-1536** (compute-representative): `ecaz corpus load
  --prefix m2_10k --profile ec_distann` of `ec_real_10k` (sha256
  `c67c5810…a35e75`), index `m2_10k_idx` built in 13.16 s.
- **Toy 400 / dim-4** (`m2_idx`): synthetic sin/cos rows, for a fast identity
  smoke.

## Commands

    # setup (committed DB)
    ecaz dev sql --db tqvector_bench … --sql "CREATE DATABASE ec_distann_m2"
    ecaz dev sql --db ec_distann_m2 … CREATE EXTENSION + m2_idx (toy)
    ecaz --database ec_distann_m2 … corpus load --prefix m2_10k --profile ec_distann …
    # tests
    ecaz dev sql --db ec_distann_m2 --file …/tc040-compare.sql       # toy identity
    ecaz dev sql --db ec_distann_m2 --file …/rtt-measure.sql         # toy RTT
    ecaz dev sql --db ec_distann_m2 --file …/tc040-rtt-10k.sql       # 10k identity + RTT

## Result: TC-040/041 — 2-node top-k identical to single-node

`ec_distann_debug_expand_search` runs the FR-081 orchestration with the
roster-selected expander (empty roster ⇒ single-node `LocalNodeExpander`;
2-node roster ⇒ `RemoteNodeExpander` = group by owner → parallel transport →
`ec_distann_expand_nodes` endpoint → position-reassemble). Cited in
`loopback-results.log`:

| corpus | rows (single/two) | set diff | ranks identical |
|--------|-------------------|----------|-----------------|
| toy 400/dim-4  | 16 / 16 | 0 / 0 | **true** |
| real 10k/dim-1536 | 12 / 12 | 0 / 0 | **true** |

Same vec_id set, same rank→vec_id mapping, `exact_dist` within 1e-6. The
remote read path is result-identical to the single-node build (FR-081-AC-1
groundwork; per-query expansion is bounded by the same BW×H the single-node
orchestration enforces).

## Result: 2-node vs 1-node latency (D4 transport-share)

Mean over N warm searches (BW=4, first-call head-graph build excluded):

| corpus | single_ms | two_node_ms | transport_delta_ms | transport_share |
|--------|-----------|-------------|--------------------|-----------------|
| toy 400/dim-4  | 0.095 | 2.997 | 2.902 | 96.8% (compute-starved) |
| real 10k/dim-1536 | ~0.69 | ~3.5 | ~2.8 | ~80% |

**D4 baton reopen trigger** ("hop RTT ≥ 50% of multinode p50"): on these small
corpora transport share exceeds 50% — but this is **not** the gate corpus.
Compute here is sub-millisecond (10k, narrow beam), while the G0 kill-check
projected ~12 ms compute at 100k / matched-recall, where the same ~3 ms
transport is ~20% share (under the trigger). So the honest M2 finding is:
**the loopback transport mechanism costs ~2.8–3.0 ms/query end-to-end**, and
the gate-relevant D4 evaluation belongs to the M4 matrix on the real
100k/dim-1536 corpus. A second, cheaper win is available: the transport
currently issues `set_config` (session identity) **per remote call**; because
the pooled connection is per-target-node, that can move to connect-time,
halving the per-hop round-trips (M2/M3 efficiency follow-up).

## Coverage notes / follow-ups

- **Materialization (transport review P1):** the debug SRF returns
  `(rank, vec_id, exact_dist)` rows directly — it does NOT feed remote hits
  through `amgettuple`, which returns a heap TID PostgreSQL fetches locally.
  `RemoteNodeExpander` is therefore deliberately **not** wired into the AM
  scan yet: remote-owned hits carry `heap_tid = INVALID` (correct for the
  FR-079 wire), so a user-facing multi-node `ORDER BY … LIMIT` needs a
  materialization tier (coordinator-visible frozen result rows keyed by
  vec_id, à la SPIRE CustomScan). That integration is scoped as the next M2/M3
  slice; M2's read-path correctness is proven here at the orchestration level.
- The FR-082 restart-on-mismatch loop (retriable epoch class → refresh + one
  restart) is M3; M2 delivers the classification (distinct SQLSTATE, wire
  round-trip tested) the restart will consume.

## Artifacts

- `loopback-results.log` — cited identity + latency lines.
- `../tc040-compare.sql`, `../rtt-measure.sql`, `../tc040-rtt-10k.sql` — the
  reproducible scripts.
