# Phase 13 SPIRE AWS Verification — operator entry points.
#
# Philosophy: Terraform brings up infra; everything else flows through
# `ecaz` commands wrapped by scripts/spire-aws/*.sh. Two commands suffice
# to stand up a topology:
#
#     make provision
#     make install-extension
#
# A full correctness pass is one command:
#
#     make pass-correctness
#
# See plan/tasks/task30-phase13b-spire-aws-verification-runbook.md.

SPIRE_AWS_DIR := $(abspath $(dir $(lastword $(MAKEFILE_LIST))))
REPO_ROOT     := $(abspath $(SPIRE_AWS_DIR)/../..)
SCRIPTS       := $(REPO_ROOT)/scripts/spire-aws

ARTIFACT_DIR ?= $(REPO_ROOT)/reviews/task-30/957-spire-aws-verification/artifacts
TOPOLOGY     ?= $(ARTIFACT_DIR)/aws-topology.json
PREFIX       ?= ec_spire_aws_synth_10k

export SPIRE_AWS_DIR REPO_ROOT ARTIFACT_DIR TOPOLOGY

.PHONY: preflight provision setup install-extension install register-remotes register \
        load-correctness load-representative load-stress \
        smoke smoke-correctness smoke-representative smoke-stress \
        bench-correctness bench-representative bench-stress \
        fault-degraded fault-strict fault-2pc fault-missing-guc fault-schema-drift fault-auth \
        single-node-smoke snapshot teardown clean \
        pass-correctness pass-representative pass-smoke \
        all-correctness all-representative

# Git ref that each EC2 node clones + cargo-pgrx-installs at bootstrap.
# Defaults to the local branch HEAD so a fresh `make pass-correctness` from
# this checkout builds the same SHA on every node. Override at the CLI for
# pinned runs:  make pass-correctness ECAZ_GIT_REF=<sha>.
ECAZ_GIT_REF ?= $(shell cd $(REPO_ROOT) && git rev-parse HEAD)
ECAZ_GIT_URL ?= https://github.com/agent-ix/ecaz.git
export ECAZ_GIT_REF ECAZ_GIT_URL

preflight:
	cd $(SPIRE_AWS_DIR) && terraform fmt -check
	cd $(SPIRE_AWS_DIR) && terraform init -backend=false
	cd $(SPIRE_AWS_DIR) && terraform validate
	cd $(REPO_ROOT) && bash -n scripts/spire-aws/*.sh
	cd $(REPO_ROOT) && if command -v shellcheck >/dev/null 2>&1; then shellcheck scripts/spire-aws/*.sh; else echo "shellcheck not found; skipping shellcheck"; fi
	cd $(REPO_ROOT) && jq empty scripts/spire-aws/suite-correctness.json scripts/spire-aws/suite-representative.json scripts/spire-aws/suite-stress.json

provision:
	@mkdir -p $(ARTIFACT_DIR)
	cd $(SPIRE_AWS_DIR) && terraform init
	cd $(SPIRE_AWS_DIR) && terraform apply -auto-approve
	cd $(SPIRE_AWS_DIR) && terraform output -json topology > $(TOPOLOGY)

setup: provision

install-extension:
	cd $(REPO_ROOT) && $(SCRIPTS)/install.sh $(TOPOLOGY) $(ARTIFACT_DIR)

install: install-extension

# All of these targets used to run from the laptop against the
# coordinator's private IP. That doesn't work — no laptop->VPC route.
# Instead, dispatch each script body on the coordinator via SSM through
# scripts/spire-aws/_run-on-coord.sh. PG is reachable via localhost on
# the coord; ecaz CLI lives at /usr/local/bin/ecaz; the repo is cloned
# to /var/lib/pgsql/build/ecaz.

ECAZ_VPC_WORK_DIR ?= /var/lib/ecaz

register-remotes:
	$(SCRIPTS)/_run-on-coord.sh register \
	  "sudo mkdir -p $(ECAZ_VPC_WORK_DIR) && sudo chown postgres:postgres $(ECAZ_VPC_WORK_DIR) && cd /var/lib/pgsql/build/ecaz && sudo -u postgres bash scripts/spire-aws/register.sh /tmp/topology.json /tmp/artifacts/register" \
	  $(TOPOLOGY) $(ARTIFACT_DIR)

register: register-remotes

load-correctness:
	$(SCRIPTS)/_run-on-coord.sh load-correctness \
	  "sudo mkdir -p $(ECAZ_VPC_WORK_DIR) && sudo chown postgres:postgres $(ECAZ_VPC_WORK_DIR) && cd /var/lib/pgsql/build/ecaz && sudo -u postgres bash scripts/spire-aws/load.sh correctness /tmp/topology.json /tmp/artifacts/load" \
	  $(TOPOLOGY) $(ARTIFACT_DIR)

load-representative:
	$(SCRIPTS)/_run-on-coord.sh load-representative \
	  "sudo mkdir -p $(ECAZ_VPC_WORK_DIR) && sudo chown postgres:postgres $(ECAZ_VPC_WORK_DIR) && cd /var/lib/pgsql/build/ecaz && sudo -u postgres bash scripts/spire-aws/load.sh representative /tmp/topology.json /tmp/artifacts/load" \
	  $(TOPOLOGY) $(ARTIFACT_DIR)

load-stress:
	$(SCRIPTS)/_run-on-coord.sh load-stress \
	  "sudo mkdir -p $(ECAZ_VPC_WORK_DIR) && sudo chown postgres:postgres $(ECAZ_VPC_WORK_DIR) && cd /var/lib/pgsql/build/ecaz && sudo -u postgres bash scripts/spire-aws/load.sh stress /tmp/topology.json /tmp/artifacts/load" \
	  $(TOPOLOGY) $(ARTIFACT_DIR)

smoke-correctness:
	$(SCRIPTS)/_run-on-coord.sh smoke-correctness \
	  "cd /var/lib/pgsql/build/ecaz && sudo -u postgres PREFIX=ec_spire_aws_synth_10k bash scripts/spire-aws/smoke.sh /tmp/topology.json /tmp/artifacts/smoke" \
	  $(TOPOLOGY) $(ARTIFACT_DIR)

smoke-representative:
	$(SCRIPTS)/_run-on-coord.sh smoke-representative \
	  "cd /var/lib/pgsql/build/ecaz && sudo -u postgres PREFIX=ec_spire_aws_repr_1m bash scripts/spire-aws/smoke.sh /tmp/topology.json /tmp/artifacts/smoke" \
	  $(TOPOLOGY) $(ARTIFACT_DIR)

smoke-stress:
	$(SCRIPTS)/_run-on-coord.sh smoke-stress \
	  "cd /var/lib/pgsql/build/ecaz && sudo -u postgres PREFIX=ec_spire_aws_synth_10m bash scripts/spire-aws/smoke.sh /tmp/topology.json /tmp/artifacts/smoke" \
	  $(TOPOLOGY) $(ARTIFACT_DIR)

bench-correctness:
	$(SCRIPTS)/_run-on-coord.sh bench-correctness \
	  "cd /var/lib/pgsql/build/ecaz && sudo -u postgres bash scripts/spire-aws/bench.sh correctness /tmp/topology.json /tmp/artifacts/bench" \
	  $(TOPOLOGY) $(ARTIFACT_DIR)

bench-representative:
	$(SCRIPTS)/_run-on-coord.sh bench-representative \
	  "cd /var/lib/pgsql/build/ecaz && sudo -u postgres bash scripts/spire-aws/bench.sh representative /tmp/topology.json /tmp/artifacts/bench" \
	  $(TOPOLOGY) $(ARTIFACT_DIR)

bench-stress:
	$(SCRIPTS)/_run-on-coord.sh bench-stress \
	  "cd /var/lib/pgsql/build/ecaz && sudo -u postgres bash scripts/spire-aws/bench.sh stress /tmp/topology.json /tmp/artifacts/bench" \
	  $(TOPOLOGY) $(ARTIFACT_DIR)

fault-degraded:
	$(SCRIPTS)/_run-on-coord.sh fault-degraded \
	  "cd /var/lib/pgsql/build/ecaz && sudo -u postgres bash scripts/spire-aws/fault.sh degraded /tmp/topology.json /tmp/artifacts/fault" \
	  $(TOPOLOGY) $(ARTIFACT_DIR)

fault-strict:
	$(SCRIPTS)/_run-on-coord.sh fault-strict \
	  "cd /var/lib/pgsql/build/ecaz && sudo -u postgres bash scripts/spire-aws/fault.sh strict /tmp/topology.json /tmp/artifacts/fault" \
	  $(TOPOLOGY) $(ARTIFACT_DIR)

fault-2pc:
	$(SCRIPTS)/_run-on-coord.sh fault-2pc \
	  "cd /var/lib/pgsql/build/ecaz && sudo -u postgres bash scripts/spire-aws/fault.sh orphaned-2pc /tmp/topology.json /tmp/artifacts/fault" \
	  $(TOPOLOGY) $(ARTIFACT_DIR)

fault-missing-guc:
	$(SCRIPTS)/_run-on-coord.sh fault-missing-guc \
	  "cd /var/lib/pgsql/build/ecaz && sudo -u postgres bash scripts/spire-aws/fault.sh missing-guc /tmp/topology.json /tmp/artifacts/fault" \
	  $(TOPOLOGY) $(ARTIFACT_DIR)

fault-schema-drift:
	$(SCRIPTS)/_run-on-coord.sh fault-schema-drift \
	  "cd /var/lib/pgsql/build/ecaz && sudo -u postgres bash scripts/spire-aws/fault.sh schema-drift /tmp/topology.json /tmp/artifacts/fault" \
	  $(TOPOLOGY) $(ARTIFACT_DIR)

fault-auth:
	$(SCRIPTS)/_run-on-coord.sh fault-auth \
	  "cd /var/lib/pgsql/build/ecaz && sudo -u postgres bash scripts/spire-aws/fault.sh auth-failure /tmp/topology.json /tmp/artifacts/fault" \
	  $(TOPOLOGY) $(ARTIFACT_DIR)

single-node-smoke:
	cd $(REPO_ROOT) && $(SCRIPTS)/single-node-smoke.sh $(TOPOLOGY) $(ARTIFACT_DIR)

# Cleanup any Secrets Manager secrets stuck in the deletion recovery
# window from a prior aborted run. Idempotent: succeeds even if no
# secrets are present. Runs as a prerequisite of `provision`.
purge-stuck-secrets:
	@REGION=$$(jq -r '.region // "us-west-2"' $(SPIRE_AWS_DIR)/terraform.tfvars 2>/dev/null || echo us-west-2); \
	  for n in ecaz-spire-aws-remote-1 ecaz-spire-aws-remote-2 ecaz-spire-aws-remote-3; do \
	    aws secretsmanager delete-secret --region $$REGION --secret-id "$$n" --force-delete-without-recovery >/dev/null 2>&1 || true; \
	  done

snapshot:
	@test -f $(TOPOLOGY) || { echo "ERROR: $(TOPOLOGY) does not exist. Run 'make provision' first." >&2; exit 1; }
	@REGION=$$(jq -r '.region' $(TOPOLOGY)); \
	  COORD_ID=$$(jq -r '.coordinator.instance_id' $(TOPOLOGY)); \
	  REMOTE_IDS=$$(jq -r '.remotes[].instance_id' $(TOPOLOGY) | tr '\n' ' '); \
	  HEAD_SHA=$$(cd $(REPO_ROOT) && git rev-parse --short HEAD); \
	  TS=$$(date -u +%Y%m%dT%H%M%SZ); \
	  echo "Snapshotting EBS root volumes for coord+remotes in $$REGION..."; \
	  for INSTANCE_ID in $$COORD_ID $$REMOTE_IDS; do \
	    VOL=$$(aws ec2 describe-instances --region $$REGION --instance-ids $$INSTANCE_ID \
	      --query 'Reservations[0].Instances[0].BlockDeviceMappings[?DeviceName==`/dev/xvda` || DeviceName==`/dev/sda1`].Ebs.VolumeId | [0]' --output text); \
	    test "$$VOL" != "None" -a -n "$$VOL" || { echo "ERROR: no root volume found for $$INSTANCE_ID" >&2; exit 1; }; \
	    SNAP=$$(aws ec2 create-snapshot --region $$REGION --volume-id $$VOL \
	      --description "spire-aws pass-correctness $$HEAD_SHA / $$INSTANCE_ID / $$TS" \
	      --tag-specifications "ResourceType=snapshot,Tags=[{Key=Phase,Value=13-spire-aws-verification},{Key=Owner,Value=kreneskyp},{Key=Packet,Value=task-30/958-spire-aws-pass-correctness},{Key=HeadSha,Value=$$HEAD_SHA},{Key=InstanceId,Value=$$INSTANCE_ID}]" \
	      --query 'SnapshotId' --output text); \
	    echo "$$INSTANCE_ID $$VOL $$SNAP" | tee -a $(ARTIFACT_DIR)/snapshots.log; \
	  done

teardown:
	cd $(SPIRE_AWS_DIR) && terraform destroy -auto-approve

clean:
	cd $(SPIRE_AWS_DIR) && rm -rf .terraform terraform.tfstate*

pass-correctness: purge-stuck-secrets provision install-extension load-correctness register-remotes smoke-correctness bench-correctness fault-degraded fault-strict snapshot teardown

# Minimal AWS smoke that bypasses the laptop->VPC connectivity gap in
# the multi-cluster register/load/smoke/bench/fault scripts. Drives a
# single-node ec_ivf + RaBitQ bench on the coordinator via SSM
# send-command, snapshots the coordinator volume, then teardown. Useful
# for proving the AWS plumbing + extension install on Graviton 4.
pass-smoke: purge-stuck-secrets provision install-extension single-node-smoke snapshot teardown

all-correctness: pass-correctness

pass-representative: purge-stuck-secrets provision install-extension load-representative register-remotes smoke-representative bench-representative fault-2pc fault-schema-drift snapshot teardown

all-representative: pass-representative
