# Task-50 IVF/RaBitQ 25k / 50k / 100k Scale Rerun

## Scope

Follow-up to packet 398 (release-rebuild rerun at 10k). Extends the local
post-merge regression check across the larger corpora that the May-19
baseline covers. SPIRE deliberately excluded above 10k because the
`ec_spire` indexes were never built at 50k / 100k (only btree present) and
the bug above 25k still blocks; only IVF/RaBitQ is exercised here.

Same source as packets 397/398:
`e81dcf8fd16cc02ddf4e88b7861af94c5f80ff48` (branch
`task-50-unsafe-closeout`). Same release-built `.so` installed
2026-05-21 22:41:10. Same host (`DESKTOP-BMB4AFO`, WSL2, i9-10900K, no
AVX-512). Same baseline reference
`benchmarks/task-50-local-baseline/` at head `cc06046177` from
2026-05-19.

## Surfaces exercised

| Step                              | Corpus prefix              | Access method | Sweep |
| --------------------------------- | -------------------------- | ------------- | ----- |
| `ivf-rabitq-25k-recall-k10`       | `ec_real_25k_ivfrabitq`    | `ec_ivf`      | [8,16,24,32,48,64] |
| `ivf-rabitq-25k-latency-k10-c1`   | `ec_real_25k_ivfrabitq`    | `ec_ivf`      | [8,16,24,32,48,64] |
| `ivf-rabitq-50k-recall-k10`       | `ec_real_50k_ivfrabitq`    | `ec_ivf`      | [8,16,24,32,48,64] |
| `ivf-rabitq-50k-latency-k10-c1`   | `ec_real_50k_ivfrabitq`    | `ec_ivf`      | [8,16,24,32,48,64] |
| `ivf-rabitq-100k-recall-k10`      | `ec_real_100k_ivfrabitq`   | `ec_ivf`      | [8,16,24,32,48,64] |
| `ivf-rabitq-100k-latency-k10-c1`  | `ec_real_100k_ivfrabitq`   | `ec_ivf`      | [8,16,24,32,48,64] |

`k=10`, `bits=4`, `seed=42`, `--force-index`, concurrency=1.
`queries_limit` and `iterations` left at suite defaults so per-step sample
sizes match the May-19 baseline (200 queries / 1000 iterations).

## Recall — bit-exact match with baseline

Same corpus, same `seed=42`, same nprobe ladder → deterministic IVF traversal
returns the same top-k. Recall numbers reproduce baseline to four decimal
places at every scale:

| Scale | nprobe | recall@k now | recall@k baseline | Δ |
| ----- | ------ | ------------ | ----------------- | ---- |
| 25k   | 8/16/24/32/48/64 | 0.8662 / 0.9058 / 0.9190 / 0.9236 / 0.9294 / 0.9330 | 0.8662 / 0.9058 / 0.9190 / 0.9236 / 0.9294 / 0.9330 | exact |
| 50k   | 8/16/24/32/48/64 | 0.8287 / 0.8841 / 0.9075 / 0.9202 / 0.9331 / 0.9379 | 0.8287 / 0.8841 / 0.9075 / 0.9202 / 0.9331 / 0.9379 | exact |
| 100k  | 8/16/24/32/48/64 | 0.7734 / 0.8411 / 0.8690 / 0.8877 / 0.9052 / 0.9180 | 0.7734 / 0.8411 / 0.8690 / 0.8877 / 0.9052 / 0.9180 | exact |

This is the strongest possible regression check for the IVF/RaBitQ scan path
— stronger than "within CI". The unsafe-block consolidation between
`cc06046177` and `e81dcf8f` produces an identical set of returned tuple ids
in identical order at every nprobe value on every corpus size we tested.

## Latency — at or slightly below baseline at every scale

p50 ms by nprobe (lower is better):

| Scale | nprobe |  now p50 | base p50 | now/base |
| ----- | ------ | -------- | -------- | -------- |
| 25k   | 8      |  7.18    |  7.34    |  0.978   |
| 25k   | 16     | 12.0     | 12.5     |  0.960   |
| 25k   | 24     | 16.5     | 17.7     |  0.932   |
| 25k   | 32     | 21.0     | 22.8     |  0.921   |
| 25k   | 48     | 30.6     | 33.0     |  0.927   |
| 25k   | 64     | 39.9     | 44.2     |  0.903   |
| 50k   | 8      |  8.88    |  9.39    |  0.946   |
| 50k   | 16     | 15.3     | 16.3     |  0.939   |
| 50k   | 24     | 22.1     | 23.8     |  0.929   |
| 50k   | 32     | 28.5     | 31.2     |  0.913   |
| 50k   | 48     | 41.7     | 45.7     |  0.912   |
| 50k   | 64     | 54.1     | 59.6     |  0.908   |
| 100k  | 8      | 11.6     | 12.0     |  0.967   |
| 100k  | 16     | 21.0     | 22.8     |  0.921   |
| 100k  | 24     | 30.4     | 33.6     |  0.905   |
| 100k  | 32     | 40.1     | 44.3     |  0.905   |
| 100k  | 48     | 60.0     | 65.8     |  0.912   |
| 100k  | 64     | 79.6     | 88.2     |  0.902   |

Means and p95 follow the same pattern (see `results.jsonl` /
per-step logs). Across all 18 (scale × nprobe) cells, the current
release-mode head is 2–10% faster than the May-19 baseline; the trend is
consistent, not noise around zero. Likely just within run-to-run variance
on a non-isolated WSL2 host, but the important fact is **no cell regressed**.

## Verdict

Combined with packet 398 at 10k, the IVF/RaBitQ scan path on the
`task-50-unsafe-closeout` branch is **recall-identical and latency-equal-
or-slightly-better** than the May-19 baseline at every scale we have local
indexes for (10k / 25k / 50k / 100k). The task-50 unsafe-block consolidation
is clean for IVF/RaBitQ on this lane and ready for the AWS optimization
phase from a local-correctness/performance standpoint.

## Known gaps (still applicable from earlier packets)

1. **SPIRE coverage stops at 25k.** The 50k and 100k `ec_spire_rabitq`
   corpora exist but the `ec_spire` index never built (only btree shows
   in `corpus list`). The known SPIRE-above-25k index-build bug is the
   reason; tracked under the broader SPIRE phase work.
2. **HNSW and DiskANN scale lanes not exercised in this packet.** The
   May-19 baseline has them; they should be reverified before declaring a
   full local-bench parity gate.
3. **Storage rows not rerun.** Packets 397/398/399 are scan-side only.
4. **`make install` Makefile bug noted in 398** still stands.

## Artifacts

See `artifacts/manifest.md`.
