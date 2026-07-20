# Initial 100k attempt disposition

The initial `candidate-bw16h25-100k` step reached ready and published
topology, then exited 1 while constructing the single-index control. The
durable runner result is `suite-manifest.json`, where 10k and 50k are
`succeeded` and 100k is `failed`.

The PostgreSQL error emitted by that attempt was:

```text
ERROR: could not extend file "base/5/622355": No space left on device
HINT: Check free disk space.
```

Only regenerable Task 179 PostgreSQL run directories were removed. The suite
was restarted with `--resume-from suite-manifest.json`, which reused 10k and
50k and reran only 100k. `suite-manifest-resume.json` is 3/3 succeeded and is
the accepted final manifest.
