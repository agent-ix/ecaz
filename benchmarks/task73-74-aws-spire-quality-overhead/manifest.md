# Task 73/74 AWS SPIRE Quality + Overhead Packet

- head SHA: `5ef85d89b5e254626d2fd3de164f719167b7e3a5`
- packet: `benchmarks/task73-74-aws-spire-quality-overhead/`
- timestamp: `2026-05-31T00:49:30Z`
- intended lane: AWS Graviton `10k-medium`
- runner: `ecaz cloud bench` driving `ecaz bench suite`
- suite config: `benchmarks/task73-74-aws-spire-quality-overhead/suite.json`
- dry-run manifest: `benchmarks/task73-74-aws-spire-quality-overhead/artifacts/suite-dry-run-manifest.json`
- result: blocked before provisioning; AWS credentials are not configured in this local environment

## Intended Matrix

The suite carries forward the M5-selected Task 73/74 points:

| surface | setting |
| --- | --- |
| SPIRE default | `top_graph_search_list_size=16`, `boundary_replica_count=0`, `nprobe=16` |
| SPIRE high recall | `top_graph_search_list_size=128`, `boundary_replica_count=0`, `nprobe=96,128` |
| IVF control | `nlists=128`, `nprobe=96,128`, `pq_fastscan`, `rerank=heap_f32`, `rerank_width=500` |

## Commands

1. Cloud status preflight:

   ```console
   ./target/debug/ecaz cloud status --profile 10k-medium --log-file benchmarks/task73-74-aws-spire-quality-overhead/artifacts/cloud-status-preflight.log
   ```

   Result from terminal stdout:

   ```text
   profile:  10k-medium
   state:    down
   cost:     ~$0.00/hr running, ~$8.00/mo retained storage
   ```

   Note: `cloud-status-preflight.log` is zero bytes because this status command's stdout was not mirrored into the log file.

2. Suite dry-run:

   ```console
   ./target/debug/ecaz bench suite run --dry-run --config benchmarks/task73-74-aws-spire-quality-overhead/suite.json --database postgres --manifest-output benchmarks/task73-74-aws-spire-quality-overhead/artifacts/suite-dry-run-manifest.json
   ```

   Result: passed and wrote `artifacts/suite-dry-run-manifest.json`.

3. Cloud bring-up attempt:

   ```console
   ./target/debug/ecaz cloud up --profile 10k-medium --git-ref 5ef85d89b --confirm-cost 8 --database postgres --log-file benchmarks/task73-74-aws-spire-quality-overhead/artifacts/cloud-up.log
   ```

   Result: failed before provisioning. Key error from `artifacts/cloud-up.log`:

   ```text
   AWS credentials are missing or invalid. Set AWS_PROFILE or run `aws configure`.
   aws: [ERROR]: An error occurred (NoCredentials): Unable to locate credentials.
   ```

4. Fresh credential check on resume:

   ```console
   aws configure list-profiles
   aws sts get-caller-identity
   ```

   Result on 2026-05-31: `aws configure list-profiles` returned no profiles, and
   `aws sts get-caller-identity` failed with `NoCredentials`. Artifact:
   `artifacts/aws-sts-get-caller-identity.log`.

## Resume Instructions

After AWS credentials are configured, rerun:

```console
./target/debug/ecaz cloud up --profile 10k-medium --git-ref 5ef85d89b --confirm-cost 8 --database postgres --log-file benchmarks/task73-74-aws-spire-quality-overhead/artifacts/cloud-up.log
./target/debug/ecaz cloud bench --profile 10k-medium --suite task73-74-aws-spire-quality-overhead --database postgres --config benchmarks/task73-74-aws-spire-quality-overhead/suite.json --ecaz-bin ./target/debug/ecaz --log-file benchmarks/task73-74-aws-spire-quality-overhead/artifacts/cloud-bench.log
```

If the host lacks `/var/lib/pgsql/18/datasets/staged-current/ec_real_100k_*.tsv`, stage/load the 100k DBPedia current dataset first, then rerun the same suite.
