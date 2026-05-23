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
        snapshot teardown clean \
        pass-correctness pass-representative all-correctness all-representative

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

register-remotes:
	cd $(REPO_ROOT) && $(SCRIPTS)/register.sh $(TOPOLOGY) $(ARTIFACT_DIR)

register: register-remotes

load-correctness:
	cd $(REPO_ROOT) && $(SCRIPTS)/load.sh correctness $(TOPOLOGY) $(ARTIFACT_DIR)

load-representative:
	cd $(REPO_ROOT) && $(SCRIPTS)/load.sh representative $(TOPOLOGY) $(ARTIFACT_DIR)

load-stress:
	cd $(REPO_ROOT) && $(SCRIPTS)/load.sh stress $(TOPOLOGY) $(ARTIFACT_DIR)

smoke:
	cd $(REPO_ROOT) && PREFIX=$(PREFIX) $(SCRIPTS)/smoke.sh $(TOPOLOGY) $(ARTIFACT_DIR)

smoke-correctness: PREFIX = ec_spire_aws_synth_10k
smoke-correctness: smoke

smoke-representative: PREFIX = ec_spire_aws_repr_1m
smoke-representative: smoke

smoke-stress: PREFIX = ec_spire_aws_synth_10m
smoke-stress: smoke

bench-correctness:
	cd $(REPO_ROOT) && $(SCRIPTS)/bench.sh correctness $(TOPOLOGY) $(ARTIFACT_DIR)

bench-representative:
	cd $(REPO_ROOT) && $(SCRIPTS)/bench.sh representative $(TOPOLOGY) $(ARTIFACT_DIR)

bench-stress:
	cd $(REPO_ROOT) && $(SCRIPTS)/bench.sh stress $(TOPOLOGY) $(ARTIFACT_DIR)

fault-degraded:
	cd $(REPO_ROOT) && $(SCRIPTS)/fault.sh degraded $(TOPOLOGY) $(ARTIFACT_DIR)

fault-strict:
	cd $(REPO_ROOT) && $(SCRIPTS)/fault.sh strict $(TOPOLOGY) $(ARTIFACT_DIR)

fault-2pc:
	cd $(REPO_ROOT) && $(SCRIPTS)/fault.sh orphaned-2pc $(TOPOLOGY) $(ARTIFACT_DIR)

fault-missing-guc:
	cd $(REPO_ROOT) && $(SCRIPTS)/fault.sh missing-guc $(TOPOLOGY) $(ARTIFACT_DIR)

fault-schema-drift:
	cd $(REPO_ROOT) && $(SCRIPTS)/fault.sh schema-drift $(TOPOLOGY) $(ARTIFACT_DIR)

fault-auth:
	cd $(REPO_ROOT) && $(SCRIPTS)/fault.sh auth-failure $(TOPOLOGY) $(ARTIFACT_DIR)

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

pass-correctness: provision install-extension register-remotes load-correctness smoke-correctness bench-correctness fault-degraded fault-strict snapshot teardown

all-correctness: pass-correctness

pass-representative: provision install-extension register-remotes load-representative smoke-representative bench-representative fault-2pc fault-schema-drift snapshot teardown

all-representative: pass-representative
