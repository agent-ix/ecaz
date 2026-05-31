# Task 73/74 AWS SPIRE Quality + Overhead Packet

- head SHA: `03e80a3d441328613efe5b4405067548ff10ccf7`
- packet: `benchmarks/task73-74-aws-spire-quality-overhead/`
- timestamp: `2026-05-31T15:05:00Z`
- lane: AWS Graviton `1m` stack (`m7g.2xlarge` DB host), after `10k-medium`
  provisioning hit the account VPC quota
- runner: `ecaz cloud bench` driving `ecaz bench suite`
- suite config: `benchmarks/task73-74-aws-spire-quality-overhead/suite.json`
- dry-run manifest: `benchmarks/task73-74-aws-spire-quality-overhead/artifacts/suite-dry-run-manifest.json`
- result: completed on AWS; artifacts synced to
  `benchmarks/task73-74-aws-spire-quality-overhead/artifacts/`

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

5. Repeated credential check on second resume:

   ```console
   aws configure list-profiles
   aws sts get-caller-identity
   ```

   Result on 2026-05-31T00:52:08Z at head
   `93295cb20ffbfa2bcc9bc6afa9a653177fa45ad7`: `aws configure
   list-profiles` returned no profiles, and `aws sts get-caller-identity`
   failed with `NoCredentials`. Artifact:
   `artifacts/aws-sts-get-caller-identity-20260531T005208Z.log`.

6. Credentials restored:

   ```console
   aws configure list-profiles
   aws sts get-caller-identity
   ```

   Result on 2026-05-31: profile `default` existed and STS returned
   `arn:aws:iam::932658697181:user/ecaz-operator`.

7. `10k-medium` resume attempt:

   ```console
   target/debug/ecaz cloud up --profile 10k-medium \
     --from-snapshot snap-0b72153293b0b749b \
     --git-ref 03e80a3d4 \
     --confirm-cost 8 \
     --database postgres \
     --log-file benchmarks/task73-74-aws-spire-quality-overhead/artifacts/cloud-up-resume-20260531.log
   ```

   Result: blocked by AWS account VPC quota:
   `VpcLimitExceeded: The maximum number of VPCs has been reached`.

8. Existing `1m` stack resume:

   ```console
   aws ec2 start-instances --region us-west-2 \
     --instance-ids i-0056e46b981edbb17 i-08b9ea039ef27adbc
   ```

   The `1m` Terraform plan was dry-run only because it would replace the
   retained stopped instances. Direct EC2 start preserved the existing volume
   and avoided allocating another VPC.

9. Remote fixture preparation:

   ```console
   aws ssm send-command ... \
     "sudo -u postgres /usr/local/bin/ecaz corpus fetch --output-dir /var/lib/pgsql/18/datasets/qdrant-dbpedia ..." \
     "sudo -u postgres /usr/local/bin/ecaz corpus prepare --profile ec_real_100k --parquet /var/lib/pgsql/18/datasets/qdrant-dbpedia/data --output-dir /var/lib/pgsql/18/datasets/staged-current ..."
   ```

   Result: generated the canonical AWS fixture files:
   `ec_real_100k_corpus.tsv` (`2.0G`), `ec_real_100k_queries.tsv` (`21M`),
   and `ec_real_100k_manifest.json` (`2.3K`). Artifacts:
   `remote-prepare-100k-20260531.json`, `corpus-fetch-100k-20260531.log`.

10. AWS benchmark:

    ```console
    target/debug/ecaz cloud bench --profile 1m \
      --suite task73-74-aws-spire-quality-overhead \
      --database postgres \
      --config benchmarks/task73-74-aws-spire-quality-overhead/suite.json \
      --ecaz-bin /usr/local/bin/ecaz \
      --log-file benchmarks/task73-74-aws-spire-quality-overhead/artifacts/cloud-bench-1m-20260531.log
    ```

    Result: completed and synced artifacts from
    `s3://ecaz-cloud-1m-b62eb804/bench-artifacts/task73-74-aws-spire-quality-overhead/20260531T145212Z/`.

11. Cost guardrail:

    ```console
    aws ec2 stop-instances --region us-west-2 \
      --instance-ids i-0056e46b981edbb17 i-08b9ea039ef27adbc
    target/debug/ecaz cloud status --profile 1m --json
    aws ec2 delete-volume --region us-west-2 --volume-id vol-05d4b734c9acb768f
    ```

    Result: profile `1m` returned to `paused`, DB instance `stopped`, estimated
    running compute `$0.00/hr`. The unattached 100 GB `10k-medium` EBS volume
    created before the VPC quota failure was deleted; verification artifact
    `cloud-cleanup-10k-medium-volume-20260531.json` shows no remaining
    `10k-medium` volumes.

## Results

| surface | setting | recall@10 | p50 | p95 | p99 |
| --- | --- | ---: | ---: | ---: | ---: |
| SPIRE default | tg16 b0 nprobe=16 | 0.8525 | 25.287 ms | 28.274 ms | 29.222 ms |
| SPIRE high recall | tg128 b0 nprobe=96 | 0.9975 | 127.618 ms | 132.383 ms | 133.764 ms |
| SPIRE ceiling | tg128 b0 nprobe=128 | 1.0000 | 162.482 ms | 162.930 ms | 163.331 ms |
| IVF control | nprobe=96, heap rerank 500 | 0.9980 | 28.6 ms | 30.2 ms | 30.9 ms |
| IVF control | nprobe=128, heap rerank 500 | 1.0000 | 35.0 ms | 36.6 ms | 37.0 ms |

The AWS result confirms the M5 local characterization: `tg128/b0` reaches the
high-recall and ceiling points, while the current default remains a fast
low-recall shape. Task 74's overhead concern also reproduces on AWS: at matched
recall, SPIRE is roughly 4.5x slower than the IVF control at nprobe 96 and
roughly 4.6x slower at nprobe 128 on this Graviton host.

## Artifacts

- Suite config: `suite.json`
- Suite manifest: `artifacts/suite-manifest.json`
- Structured results: `artifacts/results.jsonl`
- Runner log: `artifacts/suite-run.log`
- SPIRE logs:
  - `artifacts/pipeline-100k-spire-default-tg16-b0.log`
  - `artifacts/pipeline-100k-spire-highrecall-tg128-b0.log`
- IVF logs:
  - `artifacts/recall-100k-ivf-control.log`
  - `artifacts/latency-100k-ivf-control.log`
- AWS setup and preflight logs:
  - `artifacts/cloud-up-resume-20260531.log`
  - `artifacts/cloud-status-10k-medium-after-vpc-block-20260531.json`
  - `artifacts/cloud-cleanup-10k-medium-volume-20260531.json`
  - `artifacts/remote-precheck-1m-20260531.json`
  - `artifacts/remote-prepare-100k-20260531.json`
  - `artifacts/cloud-status-1m-after-stop-20260531.json`

## Notes

The original resume path targeted `10k-medium`; that path is superseded by the
completed `1m` run above because the AWS account was already at its VPC quota.
