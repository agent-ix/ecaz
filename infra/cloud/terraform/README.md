# ecaz cloud — Terraform module

Provisions a single-AZ ecaz benchmark environment on AWS:

- VPC with one private subnet and S3 + SSM VPC endpoints (no NAT gateway).
- Graviton EC2 hosts: one DB (`m7g`/`r7g`), one loader (`c7g`).
- gp3 EBS volume sized per profile, attached to the DB host as `/dev/sdf`.
- S3 bucket for parquet shards (lifecycle: 30 day expiry by default) and
  bench artifacts (retained indefinitely under a separate prefix).
- IAM role on both EC2s with SSM Session Manager and bucket R/W.

Driven from `ecaz cloud up --profile <name>`. Direct `terraform` use is
supported for review and debugging.

## Profiles

| Profile | DB instance | EBS gp3 |
|---|---|---|
| `10k`  | `m7g.large`    | 20 GB  |
| `dev`  | `m7g.large`    | 50 GB  |
| `1m`   | `m7g.xlarge`   | 100 GB |
| `10m`  | `m7g.4xlarge`  | 500 GB |
| `100m` | `r7g.4xlarge`  | 2 TB   |

## Direct usage

```sh
cd infra/cloud/terraform
terraform init
terraform plan  -var-file=profiles/10k.tfvars
terraform apply -var-file=profiles/10k.tfvars
terraform destroy -var-file=profiles/10k.tfvars
```

State is local by default; switch to a remote backend before running
multiple profiles concurrently.

## Notes

- No SSH on any host; use SSM Session Manager.
- `from_snapshot_id` restores the DB volume from a previous
  `ecaz cloud snapshot`.
- The cloud-init scripts under `cloud-init/` install Postgres 18 and
  build ecaz from `ecaz_git_ref`. They are idempotent on re-run but
  intentionally skip work if state files exist.
