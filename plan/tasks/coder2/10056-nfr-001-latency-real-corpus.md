# Task: NFR-001 Latency Lane on Real DBpedia Corpus

Motivation: `docs/RECALL_REAL_CORPUS.md:260-264` already notes that `NFR-001`
latency benchmarking reuses the same loader path as the A4 recall lane but
targets a different reporting surface (`scripts/bench_sql_latency.sh`). Now
that the real DBpedia fixture is staged and the canonical
`ec_hnsw_real_10k` / `ec_hnsw_real_50k` tables have been proven to load and
index, we can get a second axis of NFR coverage on the real corpus almost
for free — latency — without re-loading anything. This closes the gap
between "A4 is anchored on real embeddings" and "A1 is anchored on real
embeddings" which is the other half of `NFR-001` / `NFR-003` credibility.
Priority: batch 2
Status: ready

## Prompt

Wire the existing `scripts/bench_sql_latency.sh` benchmark path to the
canonical real-corpus prefixes and record a first latency sweep against the
already-loaded `ec_hnsw_real_10k` subset.

### Step 1 — read the current state

Read, in order, before touching anything:

- `scripts/bench_sql_latency.sh` — current invocation shape, assumed
  table/index names, output format.
- `docs/RECALL_REAL_CORPUS.md:147-164` — the canonical index DDL and
  schema the A4 lane uses.
- `scripts/load_real_corpus.py` — confirm the canonical corpus/queries/
  index names that get produced.
- `spec/non-functional/NFR-001-latency.md` (or whichever file currently
  houses the NFR-001 target numbers) — confirm the metric, the
  percentiles, and the query-rate assumptions the gate expects.

Do not skip this step: the bench script may already accept a prefix
argument, in which case the work is a trivial config pass-through rather
than a refactor.

### Step 2 — accept canonical prefixes directly

Modify `scripts/bench_sql_latency.sh` so it accepts a canonical real-corpus
prefix (e.g. `ec_hnsw_real_10k` or `ec_hnsw_real_50k`) as a first-class
argument and derives:

- `<prefix>_corpus` as the corpus table
- `<prefix>_queries` as the query table
- `<prefix>_m8_idx` / `<prefix>_m16_idx` as the index names

without requiring the caller to hand-edit SQL or set per-run env vars. The
existing synthetic-fixture path should keep working unchanged — add the real
path, do not replace the synthetic path.

If `bench_sql_latency.sh` currently hardcodes a corpus shape, factor the
corpus-specific bits into a small helper function or a clearly-labeled
argument group rather than duplicating the whole script. Keep the diff
minimal and focused: the goal is "accept a new fixture", not "rewrite the
bench harness".

### Step 3 — add a scratch-cluster wrapper

Add `scripts/bench_sql_latency_scratch.sh` that mirrors
`scripts/load_real_corpus_scratch.sh` in shape — same socket, port, database,
`psql` binary pinning — and forwards all other arguments to
`bench_sql_latency.sh`. This keeps the "one-shot on the pgrx scratch
cluster" story consistent with the A4 lane.

### Step 4 — record the first latency sweep

Run the new path against the already-loaded `ec_hnsw_real_10k` fixture and
record the result in the review packet. Capture at minimum:

- `m` values: `8` and `16`
- `ef_search` values: `40`, `64`, `100`, `128`, `160`, `200`
- percentiles: `p50`, `p95`, `p99`
- observed query rate (queries/sec)
- the server/client environment (scratch cluster pg17, socket path,
  `shared_buffers`, any non-default GUCs)

If `NFR-001-latency.md` declares specific percentile targets against a
specific configuration, list each measured row next to the target and mark
pass/fail. Do not move the target numbers. If the measured row misses, land
the result red and file a follow-up — the first real-corpus latency number
is worth recording even if it is below target.

### Step 5 — doc handoff

Add a new section to `docs/RECALL_REAL_CORPUS.md` titled `Reusing the
Loaded Tables for NFR-001 Latency` that points at the new script and gives
a single worked example of running it against `ec_hnsw_real_10k`. Keep the
A4 recall content unchanged. The section's job is exactly one paragraph:
"the same loaded tables serve both NFR-003 (recall) and NFR-001 (latency);
here is the latency invocation."

Cross-link from `spec/non-functional/NFR-001-latency.md` (or wherever
NFR-001 currently lives) back to the new section so future reviewers
hitting the latency doc see that the real-corpus lane exists.

## Design notes

- The scratch loader already produces the canonical
  `<prefix>_{corpus,queries}` tables and indexes. Do not re-load anything.
  If you find yourself adding a load step to the bench script, stop — the
  whole point of this task is that load and bench are decoupled on the A4
  lane, and the latency lane inherits that decoupling.
- Do not change NFR-001 target numbers or the percentiles the gate reports.
- Do not add a new bench output format. If the existing script emits CSV,
  keep CSV. If it emits plain text, keep plain text. The NFR-001 reporting
  surface is already specified; this task just retargets it.
- The latency bench should use the same `build_source_column = 'source'`
  indexes the A4 recall lane builds. Do not create a parallel index.

## Out of scope

- Rewriting `bench_sql_latency.sh` from scratch.
- Adding new latency percentiles that NFR-001 does not currently declare.
- Chasing latency regressions. Record the number, pass or fail; if it
  fails, open a review, do not try to fix it on this branch.
- Running the full 50k fixture. The 50k index build is long; start with
  10k for this task. A 50k follow-up is fine but not required.

## Validate

```bash
bash -n scripts/bench_sql_latency.sh
bash -n scripts/bench_sql_latency_scratch.sh
```

Then actually run the bench against the scratch cluster with the already-
loaded `ec_hnsw_real_10k` fixture and record the output:

```bash
scripts/bench_sql_latency_scratch.sh --prefix ec_hnsw_real_10k --m 8 --m 16 \
    --ef-search 40,64,100,128,160,200 > /tmp/nfr1_real_10k.txt
```

Attach `/tmp/nfr1_real_10k.txt` verbatim to the review packet.

Branch from current upstream main. Push branch for review.
