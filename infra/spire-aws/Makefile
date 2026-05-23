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

.PHONY: preflight package upload-tarball provision setup install-extension install register-remotes register \
        load-correctness load-representative load-stress \
        smoke smoke-correctness smoke-representative smoke-stress \
        bench-correctness bench-representative bench-stress \
        fault-degraded fault-strict fault-2pc fault-missing-guc fault-schema-drift fault-auth \
        teardown clean \
        pass-correctness pass-representative all-correctness all-representative

# Tarball produced by `make package` and consumed by `make upload-tarball`.
TARBALL ?= $(REPO_ROOT)/target/ecaz-spire-aws-latest.tar.gz

preflight:
	cd $(SPIRE_AWS_DIR) && terraform fmt -check
	cd $(SPIRE_AWS_DIR) && terraform init -backend=false
	cd $(SPIRE_AWS_DIR) && terraform validate
	cd $(REPO_ROOT) && bash -n scripts/spire-aws/*.sh
	cd $(REPO_ROOT) && if command -v shellcheck >/dev/null 2>&1; then shellcheck scripts/spire-aws/*.sh; else echo "shellcheck not found; skipping shellcheck"; fi
	cd $(REPO_ROOT) && jq empty scripts/spire-aws/suite-correctness.json scripts/spire-aws/suite-representative.json scripts/spire-aws/suite-stress.json

package:
	@mkdir -p $(REPO_ROOT)/target
	docker run --rm \
	  --platform linux/arm64 \
	  -v $(REPO_ROOT):/work \
	  -v ecaz-spire-aws-cargo-arm64:/root/.cargo \
	  -v ecaz-spire-aws-rustup-arm64:/root/.rustup \
	  -w /work \
	  amazonlinux:2023 \
	  bash /work/scripts/spire-aws/build-tarball.sh
	@ls -lh $(TARBALL)

provision:
	@mkdir -p $(ARTIFACT_DIR)
	cd $(SPIRE_AWS_DIR) && terraform init
	cd $(SPIRE_AWS_DIR) && terraform apply -auto-approve
	cd $(SPIRE_AWS_DIR) && terraform output -json topology > $(TOPOLOGY)

upload-tarball:
	@test -f $(TARBALL) || { echo "ERROR: $(TARBALL) does not exist. Run 'make package' first." >&2; exit 1; }
	@test -f $(TOPOLOGY) || { echo "ERROR: $(TOPOLOGY) does not exist. Run 'make provision' first." >&2; exit 1; }
	@BUCKET=$$(jq -r '.artifact_bucket' $(TOPOLOGY)); \
	  REGION=$$(jq -r '.region' $(TOPOLOGY)); \
	  KEY=$${ECAZ_SPIRE_AWS_TARBALL_KEY:-ecaz-latest.tar.gz}; \
	  echo "uploading $(TARBALL) to s3://$$BUCKET/$$KEY in $$REGION"; \
	  aws s3 cp $(TARBALL) "s3://$$BUCKET/$$KEY" --region "$$REGION" | tee $(ARTIFACT_DIR)/tarball-upload.log; \
	  sha256sum $(TARBALL) | tee -a $(ARTIFACT_DIR)/tarball-upload.log

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

teardown:
	cd $(SPIRE_AWS_DIR) && terraform destroy -auto-approve

clean:
	cd $(SPIRE_AWS_DIR) && rm -rf .terraform terraform.tfstate*

pass-correctness: package provision upload-tarball install-extension register-remotes load-correctness smoke-correctness bench-correctness fault-degraded fault-strict teardown

all-correctness: pass-correctness

pass-representative: package provision upload-tarball install-extension register-remotes load-representative smoke-representative bench-representative fault-2pc fault-schema-drift teardown

all-representative: pass-representative
