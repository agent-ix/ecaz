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
PACKAGE_TARBALL ?= $(ARTIFACT_DIR)/ecaz-latest.tar.gz
TUNNELED_TOPOLOGY ?= $(ARTIFACT_DIR)/aws-topology.tunneled.json
WORK_DIR ?= $(ARTIFACT_DIR)/work
ECAZ_BIN ?= $(REPO_ROOT)/target/release/ecaz
PREFIX       ?= ec_spire_aws_synth_10k
DISTRIBUTED_PLAN ?= $(ARTIFACT_DIR)/distributed-correctness/distributed-placement-plan.json

ifeq ($(filter /%,$(ARTIFACT_DIR)),)
override ARTIFACT_DIR := $(REPO_ROOT)/$(ARTIFACT_DIR)
endif
ifeq ($(filter /%,$(TOPOLOGY)),)
override TOPOLOGY := $(REPO_ROOT)/$(TOPOLOGY)
endif
ifeq ($(filter /%,$(PACKAGE_TARBALL)),)
override PACKAGE_TARBALL := $(REPO_ROOT)/$(PACKAGE_TARBALL)
endif
ifeq ($(filter /%,$(TUNNELED_TOPOLOGY)),)
override TUNNELED_TOPOLOGY := $(REPO_ROOT)/$(TUNNELED_TOPOLOGY)
endif
ifeq ($(filter /%,$(WORK_DIR)),)
override WORK_DIR := $(REPO_ROOT)/$(WORK_DIR)
endif
ifeq ($(filter /%,$(DISTRIBUTED_PLAN)),)
override DISTRIBUTED_PLAN := $(REPO_ROOT)/$(DISTRIBUTED_PLAN)
endif
override ARTIFACT_DIR := $(abspath $(ARTIFACT_DIR))
override TOPOLOGY := $(abspath $(TOPOLOGY))
override PACKAGE_TARBALL := $(abspath $(PACKAGE_TARBALL))
override TUNNELED_TOPOLOGY := $(abspath $(TUNNELED_TOPOLOGY))
override WORK_DIR := $(abspath $(WORK_DIR))
override DISTRIBUTED_PLAN := $(abspath $(DISTRIBUTED_PLAN))

export SPIRE_AWS_DIR REPO_ROOT ARTIFACT_DIR TOPOLOGY PACKAGE_TARBALL
export ECAZ_BIN WORK_DIR
export ECAZ_SPIRE_AWS_TARBALL_PATH ?= $(PACKAGE_TARBALL)

.PHONY: preflight preflight-operator preflight-state preflight-permissions preflight-representative-performance confirm-provision archive-local-state cleanup-residue provision topology setup package-extension install-extension install configure-pg-hba register-remotes register register-correctness register-representative register-stress \
        load-correctness load-representative load-stress \
        smoke smoke-correctness smoke-representative smoke-stress \
        bench-correctness bench-representative-priority bench-representative bench-representative-pooling summarize-representative-performance verify-representative-performance-summary bench-stress \
        fault-degraded fault-strict fault-2pc fault-missing-guc fault-schema-drift fault-auth \
        teardown clean \
        verify-correctness-tunneled verify-correctness-full-tunneled verify-representative-performance-tunneled verify-representative-full-tunneled plan-representative-performance pass-correctness pass-correctness-body pass-representative-performance pass-representative-performance-body pass-representative pass-representative-body all-correctness all-representative all-representative-performance

preflight:
	cd $(SPIRE_AWS_DIR) && terraform fmt -check
	cd $(SPIRE_AWS_DIR) && terraform init -backend=false
	cd $(SPIRE_AWS_DIR) && terraform validate
	cd $(REPO_ROOT) && bash -n scripts/spire-aws/*.sh
	cd $(REPO_ROOT) && if command -v shellcheck >/dev/null 2>&1; then shellcheck scripts/spire-aws/*.sh; else echo "shellcheck not found; skipping shellcheck"; fi
	cd $(REPO_ROOT) && jq empty scripts/spire-aws/suite-correctness.json scripts/spire-aws/suite-representative-priority.json scripts/spire-aws/suite-representative.json scripts/spire-aws/suite-representative-pooling.json scripts/spire-aws/suite-stress.json
	cd $(REPO_ROOT) && $(SCRIPTS)/preflight-representative-performance.sh

preflight-operator:
	cd $(REPO_ROOT) && $(SCRIPTS)/preflight-operator.sh $(SPIRE_AWS_DIR)/terraform.tfvars

preflight-state:
	cd $(REPO_ROOT) && $(SCRIPTS)/preflight-state.sh $(SPIRE_AWS_DIR)/terraform.tfstate

preflight-permissions:
	cd $(REPO_ROOT) && $(SCRIPTS)/preflight-permissions.sh

preflight-representative-performance:
	cd $(REPO_ROOT) && $(SCRIPTS)/preflight-representative-performance.sh

plan-representative-performance:
	cd $(REPO_ROOT) && $(SCRIPTS)/run-representative-performance-pass.sh --artifact-dir $(ARTIFACT_DIR)

confirm-provision:
	cd $(REPO_ROOT) && $(SCRIPTS)/confirm-provision.sh

archive-local-state:
	@mkdir -p $(ARTIFACT_DIR)
	cd $(REPO_ROOT) && $(SCRIPTS)/archive-local-state.sh $(SPIRE_AWS_DIR)/terraform.tfstate $(ARTIFACT_DIR)

cleanup-residue:
	cd $(REPO_ROOT) && $(SCRIPTS)/cleanup-residue.sh

provision: confirm-provision preflight-operator preflight-state preflight-permissions
	@mkdir -p $(ARTIFACT_DIR)
	cd $(SPIRE_AWS_DIR) && terraform init
	cd $(SPIRE_AWS_DIR) && terraform apply -auto-approve
	cd $(SPIRE_AWS_DIR) && terraform output -json topology > $(TOPOLOGY)

topology:
	@mkdir -p $(ARTIFACT_DIR)
	cd $(SPIRE_AWS_DIR) && terraform output -json topology > $(TOPOLOGY)

setup: provision

package-extension:
	cd $(REPO_ROOT) && $(SCRIPTS)/package-extension.sh $(ARTIFACT_DIR) $(PACKAGE_TARBALL)

install-extension: package-extension
	cd $(REPO_ROOT) && $(SCRIPTS)/install.sh $(TOPOLOGY) $(ARTIFACT_DIR)

install: install-extension

configure-pg-hba:
	cd $(REPO_ROOT) && $(SCRIPTS)/configure-pg-hba.sh $(TOPOLOGY) $(ARTIFACT_DIR)

register-remotes:
	cd $(REPO_ROOT) && $(SCRIPTS)/register.sh $(TOPOLOGY) $(ARTIFACT_DIR) $(DISTRIBUTED_PLAN)

register: register-remotes

register-correctness: DISTRIBUTED_PLAN = $(ARTIFACT_DIR)/distributed-correctness/distributed-placement-plan.json
register-correctness: register-remotes

register-representative: DISTRIBUTED_PLAN = $(ARTIFACT_DIR)/distributed-representative/distributed-placement-plan.json
register-representative: register-remotes

register-stress: DISTRIBUTED_PLAN = $(ARTIFACT_DIR)/distributed-stress/distributed-placement-plan.json
register-stress: register-remotes

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

verify-correctness-tunneled:
	@mkdir -p $(ARTIFACT_DIR) $(WORK_DIR)
	cd $(REPO_ROOT) && $(SCRIPTS)/with-ssm-port-forwards.sh $(TOPOLOGY) $(ARTIFACT_DIR) $(TUNNELED_TOPOLOGY) -- \
	  $(MAKE) -C $(SPIRE_AWS_DIR) ARTIFACT_DIR=$(ARTIFACT_DIR) TOPOLOGY=$(TUNNELED_TOPOLOGY) WORK_DIR=$(WORK_DIR) ECAZ_BIN=$(ECAZ_BIN) load-correctness register-correctness smoke-correctness

verify-correctness-full-tunneled:
	@mkdir -p $(ARTIFACT_DIR) $(WORK_DIR)
	cd $(REPO_ROOT) && $(SCRIPTS)/with-ssm-port-forwards.sh $(TOPOLOGY) $(ARTIFACT_DIR) $(TUNNELED_TOPOLOGY) -- \
	  $(MAKE) -C $(SPIRE_AWS_DIR) ARTIFACT_DIR=$(ARTIFACT_DIR) TOPOLOGY=$(TUNNELED_TOPOLOGY) WORK_DIR=$(WORK_DIR) ECAZ_BIN=$(ECAZ_BIN) PREFIX=ec_spire_aws_synth_10k COORD_INDEX=ec_spire_aws_synth_10k_idx load-correctness register-correctness smoke-correctness bench-correctness fault-degraded fault-strict

verify-representative-full-tunneled:
	@mkdir -p $(ARTIFACT_DIR) $(WORK_DIR)
	cd $(REPO_ROOT) && $(SCRIPTS)/with-ssm-port-forwards.sh $(TOPOLOGY) $(ARTIFACT_DIR) $(TUNNELED_TOPOLOGY) -- \
	  $(MAKE) -C $(SPIRE_AWS_DIR) ARTIFACT_DIR=$(ARTIFACT_DIR) TOPOLOGY=$(TUNNELED_TOPOLOGY) WORK_DIR=$(WORK_DIR) ECAZ_BIN=$(ECAZ_BIN) PREFIX=ec_spire_aws_repr_1m COORD_INDEX=ec_spire_aws_repr_1m_idx load-representative register-representative smoke-representative bench-representative fault-2pc fault-schema-drift

verify-representative-performance-tunneled:
	@mkdir -p $(ARTIFACT_DIR) $(WORK_DIR)
	cd $(REPO_ROOT) && $(SCRIPTS)/with-ssm-port-forwards.sh $(TOPOLOGY) $(ARTIFACT_DIR) $(TUNNELED_TOPOLOGY) -- \
	  $(MAKE) -C $(SPIRE_AWS_DIR) ARTIFACT_DIR=$(ARTIFACT_DIR) TOPOLOGY=$(TUNNELED_TOPOLOGY) WORK_DIR=$(WORK_DIR) ECAZ_BIN=$(ECAZ_BIN) PREFIX=ec_spire_aws_repr_1m COORD_INDEX=ec_spire_aws_repr_1m_idx load-representative register-representative smoke-representative bench-representative-priority bench-representative-pooling
	$(MAKE) -C $(SPIRE_AWS_DIR) ARTIFACT_DIR=$(ARTIFACT_DIR) summarize-representative-performance verify-representative-performance-summary

bench-correctness:
	cd $(REPO_ROOT) && $(SCRIPTS)/bench.sh correctness $(TOPOLOGY) $(ARTIFACT_DIR)

bench-representative-priority:
	cd $(REPO_ROOT) && $(SCRIPTS)/bench.sh representative-priority $(TOPOLOGY) $(ARTIFACT_DIR)

bench-representative:
	cd $(REPO_ROOT) && $(SCRIPTS)/bench.sh representative $(TOPOLOGY) $(ARTIFACT_DIR)

bench-representative-pooling:
	cd $(REPO_ROOT) && $(SCRIPTS)/bench.sh representative-pooling $(TOPOLOGY) $(ARTIFACT_DIR)

summarize-representative-performance:
	cd $(REPO_ROOT) && $(SCRIPTS)/summarize-representative-performance.sh $(ARTIFACT_DIR)

verify-representative-performance-summary:
	cd $(REPO_ROOT) && $(SCRIPTS)/verify-representative-performance-summary.sh $(ARTIFACT_DIR)

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

teardown:
	cd $(REPO_ROOT) && $(SCRIPTS)/empty-artifact-bucket.sh $(SPIRE_AWS_DIR)
	cd $(SPIRE_AWS_DIR) && terraform destroy -auto-approve

clean:
	cd $(SPIRE_AWS_DIR) && rm -rf .terraform terraform.tfstate*

pass-correctness:
	cd $(REPO_ROOT) && $(SCRIPTS)/run-pass-with-watchdog.sh pass-correctness-body "$(ARTIFACT_DIR)"

pass-correctness-body:
	@status=0; \
	$(MAKE) -C $(SPIRE_AWS_DIR) ARTIFACT_DIR=$(ARTIFACT_DIR) provision || status=$$?; \
	if [ $$status -eq 0 ]; then $(MAKE) -C $(SPIRE_AWS_DIR) ARTIFACT_DIR=$(ARTIFACT_DIR) install-extension || status=$$?; fi; \
	if [ $$status -eq 0 ]; then $(MAKE) -C $(SPIRE_AWS_DIR) ARTIFACT_DIR=$(ARTIFACT_DIR) verify-correctness-full-tunneled || status=$$?; fi; \
	exit $$status

all-correctness: pass-correctness

pass-representative:
	cd $(REPO_ROOT) && $(SCRIPTS)/run-pass-with-watchdog.sh pass-representative-body "$(ARTIFACT_DIR)"

pass-representative-performance:
	cd $(REPO_ROOT) && $(SCRIPTS)/run-pass-with-watchdog.sh pass-representative-performance-body "$(ARTIFACT_DIR)"

pass-representative-performance-body:
	@status=0; \
	$(MAKE) -C $(SPIRE_AWS_DIR) ARTIFACT_DIR=$(ARTIFACT_DIR) preflight-representative-performance || status=$$?; \
	if [ $$status -eq 0 ]; then $(MAKE) -C $(SPIRE_AWS_DIR) ARTIFACT_DIR=$(ARTIFACT_DIR) provision || status=$$?; fi; \
	if [ $$status -eq 0 ]; then $(MAKE) -C $(SPIRE_AWS_DIR) ARTIFACT_DIR=$(ARTIFACT_DIR) install-extension || status=$$?; fi; \
	if [ $$status -eq 0 ]; then $(MAKE) -C $(SPIRE_AWS_DIR) ARTIFACT_DIR=$(ARTIFACT_DIR) verify-representative-performance-tunneled || status=$$?; fi; \
	exit $$status

pass-representative-body:
	@status=0; \
	$(MAKE) -C $(SPIRE_AWS_DIR) ARTIFACT_DIR=$(ARTIFACT_DIR) provision || status=$$?; \
	if [ $$status -eq 0 ]; then $(MAKE) -C $(SPIRE_AWS_DIR) ARTIFACT_DIR=$(ARTIFACT_DIR) install-extension || status=$$?; fi; \
	if [ $$status -eq 0 ]; then $(MAKE) -C $(SPIRE_AWS_DIR) ARTIFACT_DIR=$(ARTIFACT_DIR) verify-representative-full-tunneled || status=$$?; fi; \
	exit $$status

all-representative: pass-representative
all-representative-performance: pass-representative-performance
