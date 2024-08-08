MAKEFLAGS += --no-print-directory

DEV_BIN_DIR := .cargo/bin
CYAN := \033[36m
RESET := \033[0m
PATH := $(DEV_BIN_DIR):$(PATH)

.PHONY: help build build.docs check commit coverage docs fix fmt lint setup test

help: ## display this help message (default task)
	@printf "%b\n" "Usage: make [$(CYAN)task$(RESET)]"
	@printf "%s\n" "Available tasks:"
	@grep -E '^[a-zA-Z_]+(\.[a-zA-Z_]+)*:.*?## .*$$' $(MAKEFILE_LIST) | \
		awk 'BEGIN {FS = ":.*?## "}; {printf "$(CYAN)%-20s$(RESET) %s\n", $$1, $$2}'

build: ## build the project
	@cargo build --workspace --all-features --all-targets --release

build.docs: ## build the documentation
	@cargo doc --no-deps --all-features

check: ## check that all files match formatting rules
	@PATH=$(PATH) dprint check

commit: ## make conventional commit
	@PATH=$(PATH) convco commit

coverage: ## generate coverage report
	@PATH=$(PATH) cargo llvm-cov --all-features report

docs: ## build & open the documentation in the browser
	@cargo doc --no-deps --open --all-features

fix: ## apply lint suggestions
	@cargo clippy --all-targets --all-features --workspace --fix

fmt: ## format all files
	@PATH=$(PATH) dprint fmt

lint: ## lint code
	@cargo clippy --all-targets --all-features --workspace

setup: ## run the setup script to install dependencies
	@./.setup.sh

test: ## run all tests
	@cargo-nextest ntr --all-features --all-targets --workspace
