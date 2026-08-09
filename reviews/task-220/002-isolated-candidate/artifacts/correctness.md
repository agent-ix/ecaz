# Task 220 MAT-16 correctness evidence

The 100k isolated run used the same immutable generation for both arms:

- generation identity: `02008577c5bff58f2044a797b5c08d0ccf8c7a5131f5d0403d630d335b2454ad9a03`
- same-generation recall pair: `mat16-control,mat16-candidate`
- materialization correctness scenarios: all passed
- scenarios covered: fewer-than-window, exactly-one-window,
  more-than-window, reject-first-window, reject-multiple-windows,
  null-payload, toasted-projection, mixed-local-remote, and
  post-first-batch-remote-failure
- representative digest equality: every eager/candidate digest pair matched;
  null and external-toast checks were true
- topology gate: `pass=true owners=3 remote_verified=2 source_rows=100000`

The complete structured runner output is `run/results.jsonl`; the compact
recall and latency logs are under `run/100k/`.
