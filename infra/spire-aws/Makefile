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
REPO_ROOT   := $(abspath $(SPIRE_AWS_DIR)/..)
SCRIPTS     := $(REPO_ROOT)/scripts/spire-aws

ARTIFACT_DIR ?= $(REPO_ROOT)/review/spire-aws-verification/artifacts
TOPOLOGY     ?= $(ARTIFACT_DIR)/aws-topology.json

export SPIRE_AWS_DIR REPO_ROOT ARTIFACT_DIR TOPOLOGY

.PHONY: setup install register \
        load-correctness load-representative load-stress \
        smoke \
        bench-correctness bench-representative bench-stress \
        fault-degraded fault-strict fault-2pc fault-missing-guc fault-schema-drift fault-auth \
        teardown clean \
        all-correctness all-representative

setup:
	@mkdir -p $(ARTIFACT_DIR)
	cd $(SPIRE_AWS_DIR) && terraform init
	cd $(SPIRE_AWS_DIR) && terraform apply -auto-approve
	cd $(SPIRE_AWS_DIR) && terraform output -json topology > $(TOPOLOGY)

install:
	$(SCRIPTS)/install.sh $(TOPOLOGY) $(ARTIFACT_DIR)

register:
	$(SCRIPTS)/register.sh $(TOPOLOGY) $(ARTIFACT_DIR)

load-correctness:
	$(SCRIPTS)/load.sh correctness $(TOPOLOGY) $(ARTIFACT_DIR)

load-representative:
	$(SCRIPTS)/load.sh representative $(TOPOLOGY) $(ARTIFACT_DIR)

load-stress:
	$(SCRIPTS)/load.sh stress $(TOPOLOGY) $(ARTIFACT_DIR)

smoke:
	$(SCRIPTS)/smoke.sh $(TOPOLOGY) $(ARTIFACT_DIR)

bench-correctness:
	$(SCRIPTS)/bench.sh correctness $(TOPOLOGY) $(ARTIFACT_DIR)

bench-representative:
	$(SCRIPTS)/bench.sh representative $(TOPOLOGY) $(ARTIFACT_DIR)

bench-stress:
	$(SCRIPTS)/bench.sh stress $(TOPOLOGY) $(ARTIFACT_DIR)

fault-degraded:
	$(SCRIPTS)/fault.sh degraded $(TOPOLOGY) $(ARTIFACT_DIR)

fault-strict:
	$(SCRIPTS)/fault.sh strict $(TOPOLOGY) $(ARTIFACT_DIR)

fault-2pc:
	$(SCRIPTS)/fault.sh orphaned-2pc $(TOPOLOGY) $(ARTIFACT_DIR)

fault-missing-guc:
	$(SCRIPTS)/fault.sh missing-guc $(TOPOLOGY) $(ARTIFACT_DIR)

fault-schema-drift:
	$(SCRIPTS)/fault.sh schema-drift $(TOPOLOGY) $(ARTIFACT_DIR)

fault-auth:
	$(SCRIPTS)/fault.sh auth-failure $(TOPOLOGY) $(ARTIFACT_DIR)

teardown:
	cd $(SPIRE_AWS_DIR) && terraform destroy -auto-approve

clean:
	cd $(SPIRE_AWS_DIR) && rm -rf .terraform terraform.tfstate*

all-correctness: setup install register load-correctness smoke bench-correctness fault-degraded fault-strict teardown

all-representative: setup install register load-representative smoke bench-representative fault-2pc fault-schema-drift teardown
