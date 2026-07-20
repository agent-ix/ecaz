# Review request: physical owner fanout A/B closeout

## Scope

Please review this isolated measurement packet for implementation commit
`5a48c7ee9` (`fix(distann): parallelize physical owner fanout`) against its
pre-change baseline `c213af204`.

Both arms use release runner `f11ffcafc` and canonical `ecaz bench suite`
configs covering the required 10k/50k/100k real-corpus scales. Every scale uses
three physical owners, graph degree 32, head index cap 4096, 20 recall queries,
10 untimed same-connection warmups, and 50 measured latency queries at
concurrency 1. A same-data single-index arm is measured inside each case as a
host/control check.

## Result

The fanout change is recall-neutral and preserves the physical storage and
topology surfaces at every scale:

| Scale | Recall baseline/candidate | Physical mean ms baseline/candidate | Mean delta | Physical p95 ms baseline/candidate | p95 delta |
| --- | ---: | ---: | ---: | ---: | ---: |
| 10k | 1.0000 / 1.0000 | 72.40 / 42.40 | -41.4% | 91.20 / 54.90 | -39.8% |
| 50k | 0.9800 / 0.9800 | 94.60 / 59.00 | -37.6% | 122.10 / 75.10 | -38.5% |
| 100k | 0.9500 / 0.9500 | 83.50 / 50.30 | -39.8% | 119.90 / 68.80 | -42.6% |

Physical generation bytes are exactly equal between arms at each scale. All
six topology gates pass with the exact source row count across three Published
owners, zero non-owned rows, zero orphans, and both remote owners verified.

The same-run single-index mean changes only +1.6%, +7.4%, and -2.5%, so the
consistent 38-41% physical mean reduction is not explained by general host
movement. See `artifacts/comparison.md` and `artifacts/manifest.md` for full
provenance and p99/storage/control details.

## Validation state

Both suite manifests report three completed steps, zero failures, zero missing
artifacts, and zero stale steps. All eighteen configured topology, recall, and
remote-engagement thresholds pass. Both configs also pass a post-run suite
audit.

This packet supplies the performance evidence deliberately deferred by packet
043. It does not close Task 179 as a whole: unrelated open findings and the
outside-review requirement remain in force.

## Requested decision

Please confirm that the A/B matrix supports accepting parallel physical owner
fanout as recall/storage neutral with a material warmed-latency improvement,
and that it closes packet 043's deferred 10k/50k/100k performance gate.
