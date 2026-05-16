# Cloud baselines — synth10k + synth50k, ec_ivf @ m8g.large (PG18)

## Purpose

Pre-optimization baselines for the IVF / RaBitQ work, captured on
the standard AWS bench harness (`infra/cloud/terraform` + `ecaz cloud`)
so post-optimization runs can re-use the same environment and
preserved EBS snapshot for direct comparison.

## Environment

| Property | Value |
|---|---|
| Region | us-west-2 (AZ us-west-2a) |
| DB instance | m8g.large (Graviton 4, 2 vCPU / 8 GB) |
| Loader instance | c8g.medium (Graviton 4, 1 vCPU / 2 GB) |
| EBS | gp3, 20 GB, 3000 IOPS, 125 MiB/s |
| OS | Amazon Linux 2023 (kernel 6.1, aarch64) |
| PostgreSQL | 18.3 (AL2023 RPM, `/usr/bin/postgres`) |
| ecaz | 0.1.1, built on the DB host via `cargo pgrx install --release` |
| Date (UTC) | 2026-05-16 |
| Profile | 10k (`infra/cloud/terraform/profiles/10k.tfvars`) |

## Datasets

Synthetic unit-sphere TSVs produced on the DB host via
`ecaz corpus generate`:

| Prefix | Rows | Dim | Queries | Generator seed (corpus / queries) |
|---|---|---|---|---|
| `synth10k` | 10,000 | 1536 | 100 | 42 / 43 |
| `synth50k` | 50,000 | 1536 | 100 | 42 / 43 |

Notes: synthetic uniform random vectors have no semantic cluster
structure, so absolute recall is lower than on real DBpedia at the
same `nprobe`. The numbers below are still useful as a *baseline*
for IVF/RaBitQ optimization because the same harness + same generator
seeds will reproduce the same inputs.

## Indexes

`ec_ivf` at default reloptions for both prefixes:

- `synth10k_idx` built in 7.92 s
- `synth50k_idx` built in 21.23 s

## Bench parameters

`ecaz bench latency` / `ecaz bench recall`:

| Parameter | Value |
|---|---|
| `k` | 10 |
| `iterations` | 200 |
| `concurrency` | 1 |
| `sweep` | profile default `nprobe = [8, 16, 24, 32, 48, 64]` |

## Results — synth10k

### Latency (200 iters, k=10, concurrency=1)

| nprobe | mean | p50 | p95 | p99 |
|---|---|---|---|---|
| 8 | 1.77 ms | 1.71 ms | 1.84 ms | 2.01 ms |
| 16 | 2.86 ms | 2.84 ms | 2.98 ms | 3.02 ms |
| 24 | 4.18 ms | 4.17 ms | 4.32 ms | 4.34 ms |
| 32 | 5.48 ms | 5.48 ms | 5.61 ms | 5.68 ms |
| 48 | 7.86 ms | 7.86 ms | 8.04 ms | 8.13 ms |
| 64 | 10.1 ms | 10.1 ms | 10.3 ms | 10.5 ms |

### Recall@10

| nprobe | recall@10 | ndcg@10 | mean q-time |
|---|---|---|---|
| 8 | 0.1440 | 0.8362 | 1.71 ms |
| 16 | 0.2550 | 0.8888 | 2.86 ms |
| 24 | 0.3650 | 0.9189 | 4.00 ms |
| 32 | 0.4580 | 0.9374 | 5.41 ms |
| 48 | 0.6010 | 0.9593 | 7.80 ms |
| 64 | 0.7290 | 0.9753 | 10.19 ms |

## Results — synth50k

### Latency (200 iters, k=10, concurrency=1)

| nprobe | mean | p50 | p95 | p99 |
|---|---|---|---|---|
| 8 | 3.55 ms | 3.52 ms | 3.81 ms | 3.86 ms |
| 16 | 6.25 ms | 6.24 ms | 6.49 ms | 6.55 ms |
| 24 | 8.91 ms | 8.89 ms | 9.18 ms | 9.28 ms |
| 32 | 11.4 ms | 11.4 ms | 11.7 ms | 11.8 ms |
| 48 | 16.7 ms | 16.8 ms | 17.1 ms | 17.2 ms |
| 64 | 21.9 ms | 21.9 ms | 22.3 ms | 22.6 ms |

### Recall@10

| nprobe | recall@10 | ndcg@10 | mean q-time |
|---|---|---|---|
| 8 | 0.0700 | 0.8047 | 3.57 ms |
| 16 | 0.1230 | 0.8506 | 6.18 ms |
| 24 | 0.1790 | 0.8803 | 8.73 ms |
| 32 | 0.2250 | 0.8977 | 11.38 ms |
| 48 | 0.3200 | 0.9219 | 16.60 ms |
| 64 | 0.4000 | 0.9394 | 21.86 ms |

## Preserved artifacts

- **EBS snapshot**: `snap-0f0806f9096f95fb7` (us-west-2, 20 GB) — contains
  the post-bench PGDATA: PG18 cluster with `tqvector_bench` database,
  `synth10k_*` + `synth50k_*` tables, and `ec_ivf` indexes. Restore via
  `ecaz cloud up --profile 10k --from-snapshot snap-0f0806f9096f95fb7`
  to skip the ~30-minute build + corpus reload.
- **Raw log files**:
  - `artifacts/m8g.large/ec_ivf/synth10k-latency.log`
  - `artifacts/m8g.large/ec_ivf/synth10k-recall.log`
  - `artifacts/m8g.large/ec_ivf/synth50k-latency.log`
  - `artifacts/m8g.large/ec_ivf/synth50k-recall.log`

## How to re-run after IVF / RaBitQ optimization

```bash
ecaz cloud up --profile 10k --from-snapshot snap-0f0806f9096f95fb7
# Optionally update the ecaz build to a newer ref:
#   ecaz cloud install --profile 10k --git-ref <sha>

# Re-run both baselines with identical flags:
ssm db host:
  ecaz bench latency --prefix synth10k --profile ec_ivf --k 10 --iterations 200 --concurrency 1
  ecaz bench recall  --prefix synth10k --profile ec_ivf --k 10
  ecaz bench latency --prefix synth50k --profile ec_ivf --k 10 --iterations 200 --concurrency 1
  ecaz bench recall  --prefix synth50k --profile ec_ivf --k 10

# Snapshot + teardown:
ecaz cloud snapshot --profile 10k --description "post-opt baselines"
ecaz cloud down --profile 10k --yes
```

## Bootstrap fixes shipped to make this work

Commits on `iam-spire-aws-operator-policy` branch (in order):

1. `4b68a59a` — add SPIRE-only IAM policy (later widened).
2. `4c7632ee` — widen prefixes to `ecaz-*`, add EBS snapshot actions.
3. `a23429e6` — fix S3 encryption IAM action names.
4. `fde6e2e5` — rightsize 10k profile to m8g.large / c8g.medium.
5. `979c40e1` — drop `parquet-tools`/`awscli2` (invalid AL2023 dnf
   package names) from cloud-init.
6. `8117b7f3` — switch PG18 paths to `/usr/bin` and install
   `postgresql18-server-devel` (AL2023 layout).
7. `b0c45c39` — use `postgresql` service unit name + systemd drop-in to
   override default PGDATA.
8. `5252dae7` — defer `shared_preload_libraries='ecaz'` until after
   pgrx build (PG can't start with a missing .so).
9. `d520b81b` — add Internet Gateway + public IPs (cloud-init needs
   github.com + sh.rustup.rs).
10. `89c39cd9` — add `ec2:*InternetGateway` IAM actions.
11. `137cfa4a` — `cargo install cargo-pgrx@^0.17` (proper SemVer).
12. `cc8bcb77` — drop bogus `--features pg18` from `cargo install`
    (the flag is on `pgrx`, not `cargo-pgrx`).
13. `a49607c6` — grant postgres NOPASSWD sudo and pass `--sudo` to
    `cargo pgrx install` so the extension files can be copied into
    root-owned `/usr/share/pgsql/extension`.

Outstanding cloud-init gap: the script installs the ecaz extension via
`cargo pgrx install` but does not build/install the `ecaz` CLI binary
itself. The CLI is required for `ecaz corpus` / `ecaz bench` on the DB
host and was built post-hoc via SSM. A follow-up should append a
`cargo build --release -p ecaz-cli && install -Dm755 .../release/ecaz
/usr/local/bin/ecaz` step at the end of `db.sh.tftpl`.
