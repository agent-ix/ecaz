# AWS Benchmark Workflow

Operational rules for the `ecaz cloud` lifecycle. Designed to make
**losing corpus + index data structurally impossible** and **rebuilding
reusable data structurally unnecessary**.

These rules are enforced both by tooling and convention:

- `ecaz cloud down` refuses to destroy a stack unless the data volume
  has a current EBS snapshot, or the caller explicitly opts out with
  `--no-snapshot-required`.
- `ecaz cloud up --from-snapshot <id>` restores PGDATA without
  re-loading corpora.
- Snapshots are listed in [snapshot inventory](#snapshot-inventory)
  with the access methods, corpora, and indexes each one covers,
  so the next round can find an exact match before paying to rebuild.

## Invariants

### 1. Never destroy without verified snapshot coverage

**Before `ecaz cloud down`, `terraform destroy`, or
`aws ec2 terminate-instances`:**

```sh
aws ec2 describe-snapshots --owner-ids self \
  --filters Name=volume-id,Values=<vol-id> Name=status,Values=completed,pending \
  --query 'sort_by(Snapshots, &StartTime) | [-1].[SnapshotId, StartTime, Description]'
```

If empty, snapshot first:

```sh
ecaz cloud snapshot --profile <p> --description '<what this captures>'
# Wait until State=completed before the destroy.
```

**Why:** the `cloud-scaling-multi-am` cycle lost a 1m IVF index
because the host was terminated before snapshotting the 150 GB data
volume. Rebuilding cost hours of compute on the next round.

### 2. Reuse the right snapshot before rebuilding anything

**Before `ecaz corpus fetch`, `corpus prepare`, or `corpus load`:**

1. Run `aws ec2 describe-snapshots --owner-ids self --query 'Snapshots[].{Id:SnapshotId,Size:VolumeSize,Started:StartTime,Desc:Description}'`.
2. Match descriptions against the [snapshot inventory](#snapshot-inventory).
3. If a snapshot covers the corpus + access method you need, restore
   with `ecaz cloud up --profile <p> --from-snapshot <id>` instead of
   loading from scratch.

**Why:** corpus fetch + slice + load + index build takes 15–30 min
at 10k–100k, 30–90 min at 1m. Re-loading the same data on every
round is pure waste.

### 3. Code changes don't require index rebuilds

Kernel changes — SIMD inner loops, LUT layouts, scoring helpers,
dispatch logic — do **not** require rebuilding indexes. After
`cargo pgrx install` + `systemctl restart postgresql`, the new
`.so` reads the same on-disk format.

**Rebuild only when on-disk format changes.** Triggers:

- `storage_format` reloption changes (`turboquant` ↔ `rabitq` ↔ `pq_fastscan`)
- `quant_bits` reloption changes (1 ↔ 2 ↔ 4 ↔ 8)
- `pq_group_size` reloption changes
- `nlists` reloption changes — list count determines centroid training,
  directory/list layout, and per-tuple assignment at build time. A new
  `nlists` value produces an entirely different IVF geometry on disk.
- `seed` or `training_sample_rows` changes that affect centroid sampling
- `MetadataPage` format version bump that affects per-tuple encoding
- `rerank` mode changes that require per-code aux state (none today)

**Don't rebuild on:**

- NEON / SVE2 / AVX2 kernel changes
- `rerank_width` default changes (scan-time knob)
- `nprobe` reloption / GUC changes (scan-time knob — chooses how many
  of the existing built lists to probe, doesn't change list contents)
- Pre-prune wiring, dedup data structures, scan dispatch fixes

When in doubt: run the scan against the existing index. If it
errors with `posting tuple length mismatch` or `RaBitQ code too
short`, then rebuild. Don't preemptively re-run `setup-*.sql`
after every code commit.

## Snapshot inventory

Owner: keep this list in sync with `aws ec2 describe-snapshots
--owner-ids self`. Update on every `ecaz cloud snapshot` invocation
and on every cycle close-out.

| Snapshot ID | Size | Started (UTC) | Covers |
| --- | --- | --- | --- |
| `snap-054feaffc50ecf1c9` | 50 GB | 2026-05-16 | real DBpedia 10k + 50k + ec_ivf (TurboQuant) indexes |
| `snap-09d29cccd558a4a47` | 250 GB | 2026-05-18 | pgvector + pgvectorscale + vchord 50k/100k/1m comparator tables |
| `snap-0f0806f9096f95fb7` | 20 GB | 2026-05-16 | synth 10k + 50k + ec_ivf (TurboQuant) |
| `snap-0bb07e0b82150a062` | 50 GB | 2026-05-22 | post-NEON round state: real 10k + 50k + 6 ec_ivf storage_format variants (TQ/RaBitQ/PQ_FASTSCAN) |
| `snap-01838d965fa09c433` | 50 GB | 2026-05-22 | post-bits=1 + rerank round state: + ec_ivf bits=1 variants + rerank=heap_f32 variants |
| `snap-0975811a1da6ea302` | 100 GB | 2026-05-22 | post-1m closure: + real DBpedia 100k + 1m loaded as `real_{100k,1m}_ivf_rabitq1_rerank_corpus` + `..._queries` + `..._rabitq_idx` (quant_bits=1, rerank=heap_f32, rerank_width=50). branch=aws-optimization-ivf-rabitq-spire HEAD=b2073ad82 |
| `snap-0e9c7743263e61d70` | 100 GB | 2026-05-22 | post-1m recall measurement, same data as snap-0975811a1da6ea302 plus the recall artifacts. Host was m8g.2xlarge by this point (resized from m8g.xlarge to fit 5.8 GB ground-truth corpus in RAM). |
| `snap-091251b06d2da2df4` | 100 GB | 2026-05-23 | post-vchord paired sweep: prior contents + `real_{50k,100k,1m}_vchord_{corpus,queries,idx}` (vchordrq lists=224/320/1024) + `gt_{50k,100k,1m}` brute-force top-10 GT tables (q=100, numpy exhaustive IP). PG `shared_preload_libraries='ecaz,vchord'`. branch=aws-optimization-ivf-rabitq-spire HEAD=7b3336b3c. Use to restore the full paired-comparator state without rebuilding vchord. |
| `snap-0e0632400184fadd4` | 100 GB | 2026-05-23 | post-Task 51 AWS IVF/RaBitQ final gate: preserved `real_1m_ivf_rabitq1_rerank_{corpus,queries}` and `real_1m_ivf_rabitq1_rerank_rabitq_idx` (`ec_ivf`, `storage_format=rabitq`, `quant_bits=1`, `rerank=heap_f32`, `rerank_width=50`), plus q=500 truth cache and suite artifacts. Host was restored as m8g.2xlarge from `snap-091251b06d2da2df4`. branch=aws-optimization-ivf-rabitq-spire HEAD=697b6d690. |
| `snap-0ac2d2a122442fd67` | 50 GB | 2026-05-24 | post-Task 55 low-cost Graviton DiskANN config audit: DBpedia/OpenAI3 source fetch plus `task55_real_{10k,100k}_diskann_{corpus,queries}` and `task55_real_{10k,100k}_diskann_idx` (`ec_diskann`, default reloptions, `bits=4`, seed=42), with suite artifacts under `benchmarks/task55-aws-diskann-lowcost-config-audit/`. Host was `10k` profile (`m8g.large`). branch main HEAD=a78a3ded7. |

**When adding rows:** include exact prefixes (`real_50k_ivf_rabitq`,
not "the 50k tables"), reloptions used (`bits=1`, `rerank=heap_f32`,
`rerank_width=50`), and the head SHA of the ecaz build at snapshot
time.

## Standard cycle

```sh
# 1. Provision (reuse existing snapshot whenever possible)
ecaz cloud up --profile 10k-medium \
              --from-snapshot snap-0bb07e0b82150a062 \
              --git-ref <branch>

# 2. Install latest extension (cheap; doesn't touch data)
ecaz cloud install --profile 10k-medium --git-ref <branch>

# 3. Bench. Indexes already present from snapshot; only build new
#    ones when storage-format reloptions actually differ.

# 4. Snapshot the post-cycle state BEFORE any destroy
ecaz cloud snapshot --profile 10k-medium \
                    --description 'post-cycle-N: <what this adds>'

# 5. Add the new snapshot ID to the inventory above with the access
#    methods + reloptions it covers.

# 6. Tear down (refuses if no snapshot present)
ecaz cloud down --profile 10k-medium --yes
```

The `ecaz cloud down` step will refuse if the data volume has no
EBS snapshot — see `crates/ecaz-cloud/src/commands/down.rs`. Override
with `--no-snapshot-required` only for genuinely disposable smoke
runs that produced no reusable corpus or index.
