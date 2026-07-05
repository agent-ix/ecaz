# 1m AWS Provisioning Blocker

- timestamp: `2026-05-30T16:25:52Z`
- profile: `1m`
- command:
  `target/debug/ecaz cloud --log-file reviews/task-67/036-scale-benchmark-100k-1m/artifacts/preflight/1m-up-confirmed.log up --profile 1m --git-ref main --confirm-cost 11 --database postgres`
- outcome:
  Terraform apply failed before a usable DB host was provisioned. The `--log-file`
  preserved the top-level Terraform apply failure in `1m-up-confirmed.log`; the
  terminal output also reported `VpcLimitExceeded: The maximum number of VPCs
  has been reached`.
- corroborating check:
  `aws ec2 describe-vpcs --region us-west-2 --query 'length(Vpcs)' --output text`
  returned `5`.
- quota lookup attempt:
  `aws service-quotas get-service-quota --service-code vpc --quota-code L-F678F1CE --region us-west-2`
  returned `AccessDeniedException` for `servicequotas:GetServiceQuota`, so the
  quota value itself could not be fetched with the current operator identity.
- cleanup:
  `target/debug/ecaz cloud --log-file reviews/task-67/036-scale-benchmark-100k-1m/artifacts/preflight/1m-down-after-vpc-limit.log down --profile 1m --yes --no-snapshot-required --database postgres`
  was run after the failed apply. Final `ecaz cloud status --profile 1m`
  reported `state: down`.
