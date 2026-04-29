# Task 28 IVF Recall Cache File Follow-Up

## Scope

Follow up packet 30148 after trying to use `--truth-cache-dir` on the 990k
width-250 recall point from packet 30136.

This packet records two things:

- the 990k width-250 recall attempt was stopped after roughly 22 minutes while
  still fetching the full corpus source matrix
- `ecaz bench recall` now has an explicit `--truth-cache-file` mode that can
  load exact truth after fetching only the query set and then fetch source rows
  only for predicted ids to preserve NDCG semantics

## 990k Attempt

Command attempted:

```text
cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 bench recall --prefix task28_ivf_pqg990k_g8_n128 --profile ec_ivf --k 100 --queries-limit 100 --sweep 40 --rerank-width 250 --force-index --truth-cache-dir review/30149-task28-ivf-990k-w250-recall-cache/artifacts/truth-cache --log-output review/30149-task28-ivf-990k-w250-recall-cache/artifacts/recall100_pqg8_990k_n128_w250_nprobe40.log
```

Observed output before stop:

```text
[recall] fetching corpus from task28_ivf_pqg990k_g8_n128_corpus ...
```

The process was active, with the Postgres backend busy on the source-table
`SELECT`, but it had not reached query fetch or exact truth by the cutoff.
The CLI process was killed; the backend exited; a follow-up PG18 `corpus list`
command succeeded.

No 990k recall result is claimed here.

## Cache-File Smoke

The explicit cache-file hit path was validated using the 10k truth artifact
from packet 30148:

```text
cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 bench recall --prefix task28_ivf_pqg10k_g8_n128 --profile ec_ivf --k 10 --queries-limit 3 --sweep 8 --rerank-width 500 --force-index --truth-cache-file review/30148-task28-ivf-recall-truth-cache-smoke/artifacts/truth-cache/truth-v1-rows10000-queries3-dim1536-k10-eb27c241304e37df.json --log-output review/30149-task28-ivf-990k-w250-recall-cache/artifacts/recall_cache_file_hit_10k.log
```

The run loaded the cache immediately after fetching queries and did not fetch
the full corpus. Result:

| recall@10 | NDCG@10 | mean q-time |
|---:|---:|---:|
| 0.8667 | 0.9934 | 70.69 ms |

## Interpretation

The original directory cache is useful after the full corpus has already been
materialized in the current run, but it does not solve cache-hit startup cost.
The explicit cache-file mode is the path to use when repeating A9/A10 recall
against an already recorded exact-truth artifact.

The remaining blocker for the 990k width-250 recall point is creating the first
990k exact-truth file without repeatedly materializing the full source table in
the CLI.

## Validation

- `cargo test -p ecaz-cli recall -- --nocapture`
  - `24 passed; 0 failed; 0 ignored`
- `git diff --check`
  - clean

## Artifacts

- `artifacts/recall_cache_file_hit_10k.log`
- `artifacts/manifest.md`
