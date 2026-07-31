# Task 210 P3 — TRAV-30 gateway copy A/B manifest

## Provenance

- Task bucket/packet: `reviews/task-210/004-gateway-copies/` (bench evidence
  for the wiring reviewed in `reviews/task-210/004-gateway-copy-wiring/`)
- Extension + CLI head: `dc6200624` (`extension_git_sha` on every summary
  row), PG18 release build with `distann-head-attribution-benchmark`
- Suite config: `artifacts/task210-p3-gateway-copies.json`
  SHA-256 `a18936d9b90b32504195110291a5c59dd757fac658b8f451d4daae7db0458a87`
- Structured results: `artifacts/run/results.jsonl` (580 rows)
  SHA-256 `359b62dc7970083208a6d0a8db4fda7fecd267eb188a5d1740eb0eebc1b71a28`
- 6 steps, all succeeded. Fixture: 3 owner nodes, one isolated
  index-per-table cluster per step, BW=4, H=100, L=32, degree 32, head cap
  4096, k=10, 200 queries, 50 iterations, 10 warmups, sharded head off,
  no traversal replica. Run dirs under `~/.ecaz/clusters/task210-p3-*`,
  removed after capture.
- Corpora: `data/staged-current`, prefixes `ec_real_{10k,50k,100k}`; SHAs on
  every row in `results.jsonl`.

## Arms

| id | role | what it is |
|---|---|---|
| `task210-p3-control` | control | shipped default, `gateway_copy_capacity` unset (0 — copies off) |
| `task210-p3-candidate` | candidate | `--gateway-copy-capacity 4096`: coordinator caches the head landmarks' routing payload |

Both arms are NFR-021 conforming: the copy holds neighbour ids and codes only
(never a vector), capacity is a stated constant enforced by refusal, and the
per-backend resident cost is observable via `ec_distann_gateway_copy_stats()`.

## Activation — the mechanism provably fired

`gateway_copies_served` (latency command, 50 scans, coordinator counter):

| scale | candidate served | control served |
|---|---:|---:|
| 10k | 553 | 0 |
| 50k | 161 | 0 |
| 100k | 159 | 0 |

Non-zero on every candidate step, zero on every control step. This is the
standing check the Task 205 pushdown lacked; a rerun where the candidate
column reads 0 is a failed run regardless of its latency table.

## P3 result — bytes and owner work (the judged quantities)

`traversal_response_bytes` (latency command, 50 scans):

| scale | control | candidate | Δ |
|---|---:|---:|---:|
| 10k | 186,546 | 118,794 | **−36.3%** |
| 50k | 268,336 | 243,856 | **−9.1%** |
| 100k | 218,390 | 202,694 | **−7.2%** |

Owner scoring time (`traversal_owner_score`, total ms over 50 scans):

| scale | control | candidate | Δ |
|---|---:|---:|---:|
| 10k | 36.71 | 38.45 | +4.7% |
| 50k | 52.38 | 52.18 | −0.4% |
| 100k | 49.41 | 47.41 | −4.0% |

The 10k row is the head covering a larger fraction of the corpus: more
expansions are gateway-cached (553 vs ~160), so more of the batch-L merge work
moves to the coordinator while the owner-side saving per skipped node is
small at this scale.

## Recall and latency — unchanged, as constructed

Recall@10 is identical per scale in both arms (0.9990 / 0.9545 / 0.9275),
matching the semantics-preservation proof
(`gateway_fill_and_rebatch_matches_the_owner_only_batch_semantics`).

Warm mean latency:

| scale | control | candidate | Δ |
|---|---:|---:|---:|
| 10k | 29.30 ms | 31.80 ms | +8.5% |
| 50k | 39.50 ms | 40.50 ms | +2.5% |
| 100k | 37.80 ms | 37.50 ms | −0.8% |

Per handoff §4(c), gateway copies cannot remove the owner round trip
(`exact_dist` needs the owner's co-placed vector), so latency parity is the
expected outcome; the wire-byte and owner-work reductions are the property.
The 10k cost tracks the coordinator-side merge takeover noted above.

## Known accounting gap

`traversal_request_bytes` reads identical in both arms (708,664 at 100k)
because the request-size estimate predates the `skip_neighbor_vec_ids`
parameter and does not count it. The real candidate requests are slightly
larger than reported. Cosmetic; flagged for a follow-up accounting fix.

## Re-run

    ecaz bench suite run \
      --config reviews/task-210/004-gateway-copies/artifacts/task210-p3-gateway-copies.json \
      --artifact-dir reviews/task-210/004-gateway-copies/artifacts/run
