MAKEFLAGS += --no-print-directory

DEV_BIN_DIR := .cargo/bin
CYAN := \033[36m
RESET := \033[0m
export PATH := $(DEV_BIN_DIR):$(PATH)

.PHONY: help build build.docs fix fmt lint setup test

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
	@dprint check

commit: ## make conventional commit
	@.cargo/bin/convco commit

docs: ## build & open the documentation in the browser
	@cargo doc --no-deps --open --all-features

fix: ## apply lint suggestions
	@cargo clippy --all-targets --all-features --workspace --fix

fmt: ## format all files
	@dprint fmt

lint: ## lint code
	@cargo clippy --all-targets --all-features --workspace

setup: ## run the setup script to install dependencies
	@./.setup.sh

test: ## run all tests
	@cargo-nextest ntr
