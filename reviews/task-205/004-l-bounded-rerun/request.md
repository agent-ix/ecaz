# Task 205 review request: bounded-L rerun

## Scope

This packet is the follow-up measurement for the bounded candidate-heap
contract. It supersedes the inert A/B in `reviews/task-205/003-ab/`. The
physical matrix is PG18, fixed BW=4, H=100, graph degree 32, head cap 4096,
head width 32, head seed count 32, top-k 10, 200 queries, 50 measured
iterations, 10 warmups, three physically sharded owner nodes, and no traversal
replica:

| arm | candidate heap limit | purpose |
|---|---:|---|
| `control-l4096` | 4096 | current-code non-binding reference |
| `candidate-l32` | 32 | BW=4/degree-32 regime-sized candidate limit |
| `sweep-l64` | 64 | short sensitivity sweep |

Each arm ran at 10k, 50k, and 100k. The implementation contract is that `L`
is a real bounded heap, `L >= max(BW,k)`, expanded/tombstoned entries do not
consume live capacity, `t` is derived from the L-th live unexpanded entry, and
each merged owner response uses `l=L` with deterministic tie retention.

## NFR-021 admissibility context

The three registrations (`task205-l4096-control`, `task205-l32-candidate`,
and `task205-l64-sweep`) are deliberately `role=context` registrations for
this measurement-only L comparison. They are evaluated using normalized
bytes-per-owned-record evidence, with fixed-roster raw growth retained only as
diagnostic output. The final structured rows show all three registrations
matching, complete across `10k,50k,100k`, and `actual_admissibility=conforming`.
The normalized growth maxima are approximately 1.095; the raw fixed-roster
maxima are approximately 11.12 and are labelled
`reported_not_threshold_fixed_roster`. No raw fixed-roster 2.0 gate is claimed
or promoted by this packet.

## Results

All physical arms had identical recall: 0.9990 at 10k, 0.9545 at 50k, and
0.9275 at 100k.

| scale | arm | latency mean (ms) | transport wait mean (ms) | response bytes/scan | pruned/scan | threshold rounds/scan | storage ratio |
|---|---|---:|---:|---:|---:|---:|---:|
| 10k | L4096 | 29.90 | 3.734 | 7,791.48 | 0.00 | 0.00 | 1.235467 |
| 10k | L32 | 28.40 | 3.033 | 3,730.92 | 163.58 | 6.46 | 1.235467 |
| 10k | L64 | 28.40 | 3.104 | 4,926.12 | 117.48 | 6.40 | 1.235467 |
| 50k | L4096 | 40.10 | 5.406 | 13,685.36 | 0.00 | 0.00 | 1.332693 |
| 50k | L32 | 39.00 | 4.463 | 5,366.72 | 336.14 | 10.30 | 1.332693 |
| 50k | L64 | 38.90 | 4.623 | 7,177.76 | 273.26 | 10.02 | 1.332693 |
| 100k | L4096 | 37.90 | 4.796 | 12,419.56 | 0.00 | 0.00 | 1.351173 |
| 100k | L32 | 37.30 | 3.996 | 4,367.80 | 341.90 | 9.78 | 1.351173 |
| 100k | L64 | 37.40 | 4.172 | 5,997.88 | 272.18 | 9.36 | 1.351147 |

Request bytes were unchanged by L: 13,669.68, 14,256.16, and 14,173.28 per
scan at 10k, 50k, and 100k respectively. Relative to L4096, L32 reduced
response bytes by 52.1%, 60.8%, and 64.8%; L64 reduced them by 36.8%, 47.5%,
and 51.7%. The lower transport-wait means track that reduction. L32 produces
more pruning than L64, as expected from the smaller live heap. The data does
not support a claim of a six-fold end-to-end win: ec_distann already transmits
scores, and the regime constrains `l` near BW/degree.

All nine storage-ratio rows are present. This rerun requests no default or
promotion change; it supplies the bounded-L implementation and the required
10k/50k/100k A/B evidence for outside review.

## Evidence

- Structured results: `artifacts/run-v2/results.jsonl`
- Final suite manifest: `artifacts/run-v2/suite-manifest.json`
- Report: `artifacts/report-v2.md`
- Run logs: `artifacts/suite-run-v2.log` and
  `artifacts/suite-run-postprocess.log`
- Per-arm summaries and latency/recall logs under
  `artifacts/run-v2/{control-l4096,candidate-l32,sweep-l64}-{10k,50k,100k}/`
