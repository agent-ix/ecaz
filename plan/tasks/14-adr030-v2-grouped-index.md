# Task 14: ADR-030 V2 Grouped Search-Code Index

Status: in progress on `adr030-v2-grouped-index`

Progress notes:
- Packet `280` ruled out "reinterpret the current scalar 4-bit code stream as grouped FastScan"
  as a credible current-format runtime path.
- Packet `287` and the kept `ADR-031` runtime prove the current format can get fast, but only to
  about `1.48-1.51ms` mean at roughly `0.8428` recall on the canonical warm real `50k` seam.
- Packet `307` and packet `309` show the current-format `ADR-032` runtime can touch about
  `0.99-1.04ms` mean near `0.842-0.852` recall, but low-`ef` recall recovery still looks
  structurally limited.
- So `ADR-030` is now the long-horizon index-v2 lane: new encoding, new hot layout, new scorer.

## Scope

Define and build a versioned tqvector index-v2 format for grouped subvector search on transformed
quantized data, with a realistic path to about `1ms` query latency and materially better recall
odds than the current scalar-code format.

## Proposed Architecture

- **Transform front-end:** support both `SRHT` and `OPQ` in the v2 metadata model; first concrete
  implementation and first feasibility spike start with `SRHT`, while `OPQ` is measured as a
  follow-on quality lever if grouped `PQ4` on `SRHT` is still short.
- **Search code:** true grouped `PQ4`, defaulting to `96` subvectors × `16` dims for the
  `1536`-dim lane, with one learned 16-centroid codebook per subvector.
- **Binary sidecar:** persisted transformed-domain sign code (`192B` at `1536` dims) kept as the
  cheap first-stage rejector because `ADR-031` already proved this lane is valuable.
- **Rerank payload:** separate higher-fidelity payload instead of forcing one code to do both jobs.
  The pragmatic first v2 shape is the existing scalar `4-bit` payload kept as a cold rerank code,
  with room for a later `PQ8`/residual rerank payload if needed.
- **Hot/cold storage split:** hot graph tuple keeps only graph-local state plus the binary sidecar
  and grouped search code; cold rerank payload lives separately so layer-0 scans do not read it
  for every candidate.
- **Query pipeline:** `binary prefilter -> grouped FastScan scorer -> tiny exact/richer rerank`.
- **Versioning:** add an explicit index-format version plus transform/layout descriptors in the
  metadata page and treat v2 as rebuild-only rather than trying to auto-upgrade v1 tuples.

## Subtasks

- [x] **Design checkpoint.** Record the v2 architecture, intended query pipeline, versioning
  story, and the "do not retry current-format grouped reinterpretation" decision.
- [ ] **Feasibility spike.** Extend the offline study harness with true grouped `PQ4` codes on
  transformed data, measuring `SRHT` first and adding an `OPQ` comparison only if needed.
- [ ] **Metadata and tuple contract.** Version the metadata page; define transform descriptors,
  codebook payload serialization, hot search tuple layout, and cold rerank payload layout.
- [ ] **Build-path training slice.** Train grouped codebooks, emit grouped search codes, emit
  binary sidecars, and emit the chosen cold rerank payload in a v2 build path.
- [ ] **Runtime search slice.** Add grouped LUT preparation and a grouped FastScan scorer on the
  hot payload, initially without broad planner/runtime rewiring.
- [ ] **Pipeline integration slice.** Rebuild the scan path around `binary -> grouped -> rerank`
  with explicit survivor budgets and measurement seams.
- [ ] **Migration and rollout.** Keep v1 readable as-is, build new indexes as v2, and document the
  rebuild requirement plus mixed-version behavior.

## Owns

- `ADR-030`
- long-horizon `FR-014` / `NFR-001` search-format redesign work

## Dependencies

- `ADR-031` prior art for binary filtering
- `ADR-032` current-format runtime measurements
- task 10 benchmark/reporting infrastructure

## Unblocks

- a real FastScan-class search-code lane
- a higher-upside path beyond the current scalar-format recall frontier
- a clean versioned migration story for future index layouts

## Deliverables

- v2 design checkpoint in `ADR-030`
- offline grouped-code feasibility study
- versioned v2 metadata and tuple layout
- grouped search-code builder
- grouped query-prep and scorer
- integrated multi-stage runtime path

## Notes

- Do not spend more time on "FastScan over today's scalar code bytes" unless a new, concrete
  reason appears.
- The first risk to kill is not SIMD mechanics. It is whether true grouped `PQ4` on transformed
  tqvector data has the ranking quality to justify the new format.
- If the grouped search code is promising, the intended steady-state architecture is:
  `binary prefilter + true grouped FastScan search code + tiny rerank`.
