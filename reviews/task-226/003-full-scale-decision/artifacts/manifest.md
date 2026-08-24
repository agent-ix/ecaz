# Task 226 packet 003 artifact manifest

- Task bucket / packet: `reviews/task-226/003-full-scale-decision/`
- Code correction SHA: `c85196ce841c1cbcea187dbefb3c10430fb611be`
- Clean release execution head: `a1f1584966011ca7c16175fe91f8efc302c8cf25`
- Suite runner binary SHA: `b54f321a579ccdac1535aedc4e3387f78811b0af`
- Lane: three-owner physical PG18 release extension, fixed 4,096 persisted
  sharded head, L32, H100, RaBitQ, lazy-10, 200 held-out queries, top-k 10
- Matrix: fresh 10k and 50k fixtures in this packet plus the immutable 100k
  production result in `reviews/task-226/002-current-head-100k/`
- Isolation: one immutable generation per scale shared only by BW4 control and
  BW8 candidate; no post-lifecycle fixture reuse
- SuiteConfig SHA-256:
  `876b63c99280911f1d8e7cb837ed379eeacbf806056d18230c9bc0ca3b5a751a`
- Run directories: `/home/peter/.ecaz/clusters/task226-bw8-full-10k` and
  `/home/peter/.ecaz/clusters/task226-bw8-full-50k`; both are outside the repo
  and will be removed after packet-local evidence capture
- Storage format / rerank: unchanged physical RaBitQ generation; no format or
  rerank-mode change

## Preregistered evidence

- `task226-bw8-full-10k-50k.json`: release production SuiteConfig for the two
  remaining scales.
- `suite-audit.log`: config audit passes with exactly two steps.
- `suite-dry-run.log` and `suite-dry-run-manifest.json`: expanded commands
  prove fresh external run directories and a BW4/BW8-only beam-width delta.
- Packet 002 100k gate: recall 0.9285 to 0.9450, paired delta +0.0165 with
  95% CI +0.0080 to +0.0265; warm mean 16.4 to 16.2 ms; p95 19.0 to 19.8 ms;
  A/A byte-identical.

Suite audit, dry-run expansion, run manifests/results, compact decision lines,
commands, timestamps, artifact hashes, and fixture cleanup will be appended as
the matrix executes.
