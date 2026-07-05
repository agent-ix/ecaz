# Review Request: SPIRE Phase 13e Quota-Fit Correctness Topology

Requester: coder1
Date: 2026-05-25
Head SHA: `9f097000b5036d1b472844bb56ea3690864e02f4`
Review focus: verify the SPIRE AWS correctness tier now follows a quota-fit Graviton topology and no longer re-arms the 40-vCPU failure from packet 975.

## Summary

This slice addresses the blocking reviewer finding on packet 976: the previous Graviton defaults used 40 vCPU and would still fail the observed 16-vCPU account limit.

Changes:

- Correctness-tier Terraform defaults now use `m7g.large` for the coordinator and each of the three remotes, for 8 vCPU total.
- Representative/stress tiers remain documented as the larger `r7g.4xlarge` + 3 x `r7g.2xlarge` topology, but only after separate 40-vCPU quota proof.
- Operator preflight defaults now match the quota-fit correctness topology.
- `bootstrap-node.sh` now chooses PostgreSQL memory GUCs from actual host RAM, so an 8 GiB correctness host no longer receives `shared_buffers = 32GB`.
- Phase 13a, Phase 13b, and the parent AWS verification packet document the split between correctness and representative/stress topology.

No AWS resources were provisioned for this packet.

## Validation

- `bash -n scripts/spire-aws/preflight-operator.sh scripts/spire-aws/bootstrap-node.sh` passed.
- `terraform -chdir=infra/spire-aws fmt -check` passed.
- `terraform -chdir=infra/spire-aws validate` passed.
- A mocked arm64 AMI with no explicit instance overrides uses `m7g.large` coordinator/remotes and passes.
- A mocked `r6i.4xlarge` coordinator is rejected.
- `git diff --check` passed.

See `artifacts/manifest.md` for packet-local logs.

## Notes

This does not claim AWS correctness. It removes the quota/sizing blocker identified after 975 so the next correctness attempt can use the checked-in Graviton runbook without inventing a new hardware shape.
