# Variables
# -----------------------------------------------------------------------------
# Tool versions from Cargo.toml [workspace.metadata.tools]
CARGO_AUDIT_VERSION := $(shell bash scripts/cargo-tool-version.sh cargo-audit)
CARGO_DENY_VERSION := $(shell bash scripts/cargo-tool-version.sh cargo-deny)
CARGO_EDIT_VERSION := $(shell bash scripts/cargo-tool-version.sh cargo-edit)
CARGO_FUZZ_VERSION := $(shell bash scripts/cargo-tool-version.sh cargo-fuzz)
CARGO_LLVM_COV_VERSION := $(shell bash scripts/cargo-tool-version.sh cargo-llvm-cov)
CARGO_MACHETE_VERSION := $(shell bash scripts/cargo-tool-version.sh cargo-machete)
CARGO_NEXTEST_VERSION := $(shell bash scripts/cargo-tool-version.sh cargo-nextest)
CARGO_VET_VERSION := $(shell bash scripts/cargo-tool-version.sh cargo-vet)
FLAMEGRAPH_VERSION := $(shell bash scripts/cargo-tool-version.sh flamegraph)
LYCHEE_VERSION := $(shell bash scripts/cargo-tool-version.sh lychee)
# Tool versions from tools.toml
PREK_VERSION := $(shell bash scripts/tool-version.sh prek)

V = 0  # 0 / 1 - verbose mode
Q = $(if $(filter 1,$V),,@) # Quiet mode, suppress command output
M = $(shell printf "\033[0;34m>\033[0m") # Message prefix for commands

# Verbose options for specific targets (defaults to true, can be overridden)
VERBOSE ?= true

# TARGET_DIR controls where cargo places build artifacts.
# Can be overridden to use a separate directory.
TARGET_DIR ?= target

# Compiler configuration
# Uses clang by default (required by ed25519-blake2b and other deps).
# When sccache is available, wraps the compiler for build caching.
# Set CARGO_INCREMENTAL=0 with sccache for better cache hit rates.
# To disable sccache: make build SCCACHE=
SCCACHE ?= $(shell command -v sccache 2>/dev/null)

ifeq ($(SCCACHE),)
CC ?= clang
CXX ?= clang++
else
CC ?= sccache clang
CXX ?= sccache clang++
RUSTC_WRAPPER ?= sccache
CARGO_INCREMENTAL ?= 0
export RUSTC_WRAPPER
export CARGO_INCREMENTAL
endif

export CC
export CXX

# FAIL_FAST controls whether `cargo nextest` should stop after the first test
# failure. When set to `true` the `--no-fail-fast` flag is omitted so tests
# abort on the first failure. When `false` (the default) the flag is included
# allowing the full test suite to run.
FAIL_FAST ?= false

# NEXTEST_PROFILE selects the nextest profile from .config/nextest.toml.
# CI should set NEXTEST_PROFILE=ci to limit parallelism on resource-constrained runners.
NEXTEST_PROFILE ?= default

# CARGO_CI_PROFILE selects the Cargo compile profile used by nextest.
CARGO_CI_PROFILE ?= nextest

# Select the appropriate flag for `cargo nextest` depending on FAIL_FAST.
ifeq ($(FAIL_FAST),true)
FAIL_FAST_FLAG :=
else
FAIL_FAST_FLAG := --no-fail-fast
endif

# EXTRA_FEATURES allows adding optional features to cargo builds/tests.
# Can be set directly: make cargo-test EXTRA_FEATURES="hypersync"
# Or use convenience flags below for backwards compatibility.
EXTRA_FEATURES ?=

# HYPERSYNC is a convenience flag that adds hypersync to EXTRA_FEATURES.
# Can be overridden: make check-code HYPERSYNC=true
HYPERSYNC ?= false
ifeq ($(HYPERSYNC),true)
EXTRA_FEATURES += hypersync
endif

# DEFI controls whether defi feature is included (default: true).
# Can be disabled: make cargo-test-core DEFI=false
DEFI ?= true
ifeq ($(DEFI),true)
BASE_FEATURES := arrow,ffi,high-precision,streaming,defi
else
BASE_FEATURES := arrow,ffi,high-precision,streaming
endif

# Combine base features with extra features
ifneq ($(strip $(EXTRA_FEATURES)),)
CARGO_FEATURES := $(BASE_FEATURES),$(EXTRA_FEATURES)
else
CARGO_FEATURES := $(BASE_FEATURES)
endif

# Core crates (excludes adapters/* and nautilus-cli)
CORE_CRATES := nautilus-analysis nautilus-backtest nautilus-common nautilus-core \
    nautilus-cryptography nautilus-data nautilus-execution nautilus-indicators \
    nautilus-infrastructure nautilus-live nautilus-model nautilus-network \
    nautilus-persistence nautilus-portfolio nautilus-risk nautilus-serialization \
    nautilus-system nautilus-testkit nautilus-trading

# Adapter crates (crates/adapters/*)
ADAPTER_CRATES := nautilus-architect-ax nautilus-betfair nautilus-binance \
    nautilus-bitmex nautilus-blockchain nautilus-bybit nautilus-databento \
    nautilus-deribit nautilus-dydx nautilus-hyperliquid nautilus-kraken \
    nautilus-okx nautilus-polymarket nautilus-sandbox nautilus-tardis

# > Colors
# Use ANSI escape codes directly for cross-platform compatibility (Git Bash on Windows doesn't have tput)
RED    := \033[0;31m
GREEN  := \033[0;32m
YELLOW := \033[0;33m
BLUE   := \033[0;34m
PURPLE := \033[0;35m
CYAN   := \033[0;36m
GRAY   := \033[0;37m
RESET  := \033[0m

.DEFAULT_GOAL := help

# Requires GNU Make across all platforms (Windows users should install it via MSYS2 or WSL).

#== Installation

.PHONY: install-deps
install-deps:  #-- Fetch Rust workspace dependencies
	$(info $(M) Fetching Rust workspace dependencies...)
	$Q cargo fetch --locked

.PHONY: install
install:
install: export BUILD_MODE=release
install:  #-- Install Rust CLI in release mode with helper dependencies
	$(info $(M) Installing Rust CLI in release mode...)
	$Q $(MAKE) --no-print-directory install-cli

.PHONY: install-debug
install-debug:
install-debug: export BUILD_MODE=debug
install-debug:  #-- Install in debug mode for development
	$(info $(M) Building Rust workspace in debug mode...)
	$Q cargo build --workspace --features "$(CARGO_FEATURES)"

#== Build

.PHONY: build
build: cargo-build
build:  #-- Build the package in release mode

.PHONY: build-debug
build-debug:  #-- Build the package in debug mode (recommended for development)
	$(info $(M) Building Rust workspace in debug mode...)
	cargo build --workspace --features "$(CARGO_FEATURES)"

#== Clean

CLEAN_BUILD_OUTPUTS := target target-v2 build dist .coverage*
CLEAN_BUILD_OUTPUTS += .benchmarks*
CLEAN_GENERATED_OUTPUTS := release-publication-evidence graphify-out
CLEAN_PROTECTED_OUTPUTS := .codex .agentflow .understand-anything project.html
CLEAN_PROTECTED_OUTPUTS += tests/test_data/large tests/test_data/local

.PHONY: clean-dry-run
clean-dry-run:  #-- List reproducible build outputs make clean would remove
	@echo "Reproducible build outputs selected for cleanup:"
	@found=0; \
	for candidate in $(CLEAN_BUILD_OUTPUTS); do \
		if [ -e "$$candidate" ]; then \
			printf "  %s\n" "$$candidate"; \
			found=1; \
		fi; \
	done; \
	if [ "$$found" -eq 0 ]; then echo "  (none)"; fi

.PHONY: clean
clean:  #-- Remove only documented reproducible build outputs
	$(info $(M) Removing reproducible build outputs...)
	$Q rm -rf -- $(CLEAN_BUILD_OUTPUTS)

.PHONY: clean-generated-dry-run
clean-generated-dry-run:  #-- List generated audit and analysis outputs
	@echo "Generated audit and analysis outputs selected for cleanup:"
	@found=0; \
	for candidate in $(CLEAN_GENERATED_OUTPUTS); do \
		if [ -e "$$candidate" ]; then \
			printf "  %s\n" "$$candidate"; \
			found=1; \
		fi; \
	done; \
	if [ "$$found" -eq 0 ]; then echo "  (none)"; fi

.PHONY: clean-generated
clean-generated:  #-- Remove generated audit and analysis outputs (requires FORCE=1)
	@if [ "$$FORCE" != "1" ]; then \
		echo "Refusing generated-output cleanup; run make clean-generated FORCE=1"; \
		exit 1; \
	fi
	$(info $(M) Removing generated audit and analysis outputs...)
	$Q rm -rf -- $(CLEAN_GENERATED_OUTPUTS)

.PHONY: distclean-dry-run
distclean-dry-run:  #-- List the complete guarded cleanup set without deleting
	@$(MAKE) --no-print-directory clean-dry-run
	@$(MAKE) --no-print-directory clean-generated-dry-run
	@echo "Protected local state is never selected:"
	@for candidate in $(CLEAN_PROTECTED_OUTPUTS); do printf "  %s\n" "$$candidate"; done

.PHONY: distclean
distclean:  #-- Remove build and generated outputs only (requires FORCE=1)
	@if [ "$$FORCE" != "1" ]; then \
		echo "Refusing guarded cleanup; inspect make distclean-dry-run, then pass FORCE=1"; \
		exit 1; \
	fi
	@$(MAKE) --no-print-directory clean
	@$(MAKE) --no-print-directory clean-generated FORCE=1

.PHONY: ib-stop
ib-stop:  #-- Stop local TWS/IBC processes and Docker IB Gateway containers
	@echo "Stopping local TWS/IBC processes..."
	@pkill -TERM -f "Trader Workstation" || true
	@pkill -TERM -f "ibcstart.sh" || true
	@pkill -TERM -f "displaybannerandlaunch.sh" || true
	@echo "Stopping Docker IB Gateway containers..."
	@docker ps --format '{{.Names}} {{.Image}}' | awk '/ib-gateway|ibgateway|Trader Workstation|tws/ {print $$1}' | xargs -r docker stop >/dev/null 2>&1 || true
	@sleep 2
	@pkill -KILL -f "Trader Workstation" || true
	@pkill -KILL -f "ibcstart.sh" || true
	@pkill -KILL -f "displaybannerandlaunch.sh" || true
	@docker ps --format '{{.Names}} {{.Image}}' | awk '/ib-gateway|ibgateway|Trader Workstation|tws/ {print $$1}' | xargs -r docker kill >/dev/null 2>&1 || true
	@echo "Done."

#== Code Quality

.PHONY: format
format:  #-- Format Rust with nightly rustfmt
	cargo +nightly fmt

.PHONY: pre-commit
pre-commit:  #-- Run all pre-commit hooks on all files
	prek run --all-files

# The check-code target uses CARGO_FEATURES which is controlled by the HYPERSYNC flag.
# By default, hypersync is excluded to speed up checks. Override with: make check-code HYPERSYNC=true
.PHONY: check-code
check-code:  #-- Run clippy on lib/test targets (use HYPERSYNC=true to include hypersync feature)
	$(info $(M) Running code quality checks...)
	@cargo clippy --workspace --lib --tests --features "$(CARGO_FEATURES)" --profile nextest -- -D warnings
	@printf "$(GREEN)Checks passed$(RESET)\n"

.PHONY: check-all-targets
check-all-targets:  #-- Run clippy on all targets including bins and examples (nightly)
	$(info $(M) Running full clippy on all targets...)
	@cargo clippy --workspace --all-targets --features "$(CARGO_FEATURES),examples" --profile nextest -- -D warnings
	@printf "$(GREEN)All-targets check passed$(RESET)\n"

# Time a block of make sub-targets. Use as:
#   @$(timer_start) \
#       $(MAKE) ... \
#       && $(MAKE) ... \
#   $(call timer_end,Time label)
# Prints "<Time label> time: H:MM:SS" and propagates the block's exit code.
timer_start = _t_start=$$(date +%s); (

define timer_end
); _t_rc=$$?; \
_t_elapsed=$$(( $$(date +%s) - _t_start )); \
printf "$(1) time: %d:%02d:%02d\n" $$(( _t_elapsed / 3600 )) $$(( (_t_elapsed % 3600) / 60 )) $$(( _t_elapsed % 60 )); \
exit $$_t_rc
endef

.PHONY: pre-flight
pre-flight: export CARGO_TARGET_DIR=$(TARGET_DIR)
pre-flight:  #-- Run Rust-only pre-flight checks (format, check-code, cargo-test, cargo-build)
	$(info $(M) Running pre-flight checks...)
	@if ! git diff --quiet; then \
		printf "$(RED)ERROR: You have unstaged changes$(RESET)\n"; \
		printf "$(YELLOW)Stage your changes first:$(RESET) git add .\n"; \
		exit 1; \
	fi
	@$(timer_start) \
		$(MAKE) --no-print-directory format \
		&& $(MAKE) --no-print-directory check-code EXTRA_FEATURES="hypersync" \
		&& $(MAKE) --no-print-directory cargo-test-extras \
		&& $(MAKE) --no-print-directory cargo-build \
		&& $(MAKE) --no-print-directory security-audit \
	$(call timer_end,Pre-flight)

.PHONY: clippy
clippy:  #-- Run clippy linter (check only, workspace lints)
	cargo clippy --all-targets --all-features -- -D warnings

.PHONY: clippy-fix
clippy-fix:  #-- Run clippy linter with automatic fixes (workspace lints)
	cargo clippy --fix --all-targets --all-features --allow-dirty --allow-staged -- -D warnings

.PHONY: clippy-fix-nightly
clippy-fix-nightly:  #-- Run clippy linter with nightly toolchain and automatic fixes (workspace lints + additional strictness)
	cargo +nightly clippy --fix --all-targets --all-features --allow-dirty --allow-staged -- -D warnings

.PHONY: clippy-pedantic-crate-%
clippy-pedantic-crate-%:  #-- Run clippy linter for a specific Rust crate (usage: make clippy-crate-<crate_name>)
	cargo clippy --all-targets --all-features -p $* -- -D warnings \
		-W clippy::todo \
		-W clippy::unwrap_used \
		-W clippy::expect_used

#== Dependencies

.PHONY: outdated
outdated: check-edit-installed  #-- Check for outdated dependencies
	cargo upgrade --dry-run --incompatible
	@printf "\n$(CYAN)Checking tool versions...$(RESET)\n"
	@outdated_count=0; \
	for tool in cargo-audit:$(CARGO_AUDIT_VERSION) cargo-deny:$(CARGO_DENY_VERSION) cargo-edit:$(CARGO_EDIT_VERSION) cargo-fuzz:$(CARGO_FUZZ_VERSION) cargo-llvm-cov:$(CARGO_LLVM_COV_VERSION) cargo-machete:$(CARGO_MACHETE_VERSION) cargo-nextest:$(CARGO_NEXTEST_VERSION) cargo-vet:$(CARGO_VET_VERSION) flamegraph:$(FLAMEGRAPH_VERSION) lychee:$(LYCHEE_VERSION); do \
		name=$${tool%%:*}; current=$${tool##*:}; \
		latest=$$(cargo search $$name --limit 1 2>/dev/null | head -1 | awk -F\" '{print $$2}'); \
		if [ "$$current" != "$$latest" ]; then \
			printf "$(YELLOW)  $$name: $$current → $$latest$(RESET)\n"; \
			outdated_count=$$((outdated_count + 1)); \
		fi; \
	done; \
	[ $$outdated_count -eq 0 ] && printf "$(GREEN)  All tools up to date ✓$(RESET)\n"

.PHONY: update
update: cargo-update  #-- Update Rust dependencies

.PHONY: install-tools
install-tools: check-binstall-installed  #-- Install required development tools
	cargo install cargo-deny --version $(CARGO_DENY_VERSION) --locked \
	&& cargo install cargo-edit --version $(CARGO_EDIT_VERSION) --locked \
	&& cargo install cargo-fuzz --version $(CARGO_FUZZ_VERSION) --locked \
	&& cargo install cargo-machete --version $(CARGO_MACHETE_VERSION) --locked \
	&& cargo install cargo-nextest --version $(CARGO_NEXTEST_VERSION) --locked \
	&& cargo install cargo-llvm-cov --version $(CARGO_LLVM_COV_VERSION) --locked \
	&& cargo install cargo-audit --version $(CARGO_AUDIT_VERSION) --locked \
	&& cargo install cargo-vet --version $(CARGO_VET_VERSION) --locked \
	&& cargo install flamegraph --version $(FLAMEGRAPH_VERSION) --locked \
	&& cargo install lychee --version $(LYCHEE_VERSION) --locked \
	&& cargo binstall prek --version $(PREK_VERSION) --no-confirm --locked \
	&& bash scripts/install-osv-scanner.sh

#== Security

# Run an audit step: capture stdout+stderr, only display on failure.
# Args: $(1) display name, $(2) command to run.
define audit_step
	printf "$(CYAN)Running $(1)...$(RESET) "; \
	if _out=$$($(2) 2>&1); then \
		printf "$(GREEN)ok$(RESET)\n"; \
	else \
		rc=$$?; printf "$(RED)failed$(RESET)\n%s\n" "$$_out"; exit $$rc; \
	fi
endef

.PHONY: security-audit
security-audit: check-audit-installed check-deny-installed check-vet-installed check-osv-scanner-installed  #-- Run Rust supply-chain audit (cargo-audit, cargo-deny, cargo-vet, osv-scanner)
	$(info $(M) Running security audit...)
	@$(call audit_step,cargo audit,cargo audit --color never)
	@$(call audit_step,cargo deny,cargo deny --all-features check advisories licenses sources bans)
	@$(call audit_step,cargo vet,cargo vet --locked)
	@$(call audit_step,osv-scanner,osv-scanner --config=osv-scanner.toml --lockfile=Cargo.lock)

.PHONY: cargo-deny
cargo-deny: check-deny-installed  #-- Run cargo-deny checks (advisories, sources, bans, licenses)
	cargo deny --all-features check

.PHONY: cargo-vet
cargo-vet: check-vet-installed  #-- Run cargo-vet supply chain audit
	cargo vet

#== Documentation

.PHONY: docs
docs: docs-rust docs-check-links  #-- Build Rust docs and validate supported docs/examples

.PHONY: docs-rust
docs-rust:  #-- Build Rust documentation with cargo doc
	cargo doc --all-features --no-deps --workspace

.PHONY: docsrs-check
docsrs-check: export DOCS_RS=1
docsrs-check: export RUSTDOCFLAGS=--cfg docsrs -D warnings
docsrs-check: check-hack-installed #-- Check documentation builds for docs.rs compatibility
	cargo +nightly hack --workspace doc --no-deps --all-features

.PHONY: docs-check-links
docs-check-links:  #-- Check supported local docs/examples authority and links
	$(info $(M) Checking supported documentation and examples...)
	@scripts/ai/check_docs_examples_governance.sh
	@printf "$(GREEN)Local docs/examples check passed$(RESET)\n"

.PHONY: docs-check-external-links
docs-check-external-links:  #-- Check external documentation links (periodic network audit)
	$(info $(M) Checking external documentation links...)
	@lychee \
		--verbose \
		--no-progress \
		--exclude-all-private \
		--max-retries 3 \
		--retry-wait-time 5 \
		--timeout 30 \
		--max-concurrency 10 \
		--accept "100..=103,200..=299,429,502..=504" \
		--include-fragments \
		--fallback-extensions md,py,html \
		--exclude-path target \
		--exclude-file .lycheeignore \
		"**/*.md" "docs/**/*.py"
	@printf "$(GREEN)External link check passed$(RESET)\n"

#== Rust Development

.PHONY: cargo-build
cargo-build:  #-- Build Rust crates in release mode
	cargo build --release --all-features

.PHONY: cargo-update
cargo-update:  #-- Update Rust dependencies (versions from Cargo.toml)
	cargo update

.PHONY: cargo-check
cargo-check:  #-- Check Rust code without building
	cargo check --workspace --all-features

# Security tool checks
.PHONY: check-audit-installed
check-audit-installed:  #-- Verify cargo-audit is installed
	@if ! cargo audit --version >/dev/null 2>&1; then \
		echo "cargo-audit is not installed. You can install it using 'cargo install cargo-audit'"; \
		exit 1; \
	fi

.PHONY: check-deny-installed
check-deny-installed:  #-- Verify cargo-deny is installed
	@if ! cargo deny --version >/dev/null 2>&1; then \
		echo "cargo-deny is not installed. You can install it using 'cargo install cargo-deny'"; \
		exit 1; \
	fi

.PHONY: check-binstall-installed
check-binstall-installed:  #-- Verify cargo-binstall is installed (one-off prerequisite for install-tools)
	@if ! command -v cargo-binstall >/dev/null 2>&1; then \
		printf "$(YELLOW)cargo-binstall is required but not installed$(RESET)\n"; \
		printf "Install once per machine with: $(CYAN)cargo install cargo-binstall --locked$(RESET)\n"; \
		printf "See: https://github.com/cargo-bins/cargo-binstall\n"; \
		exit 1; \
	fi

.PHONY: check-vet-installed
check-vet-installed:  #-- Verify cargo-vet is installed
	@if ! cargo vet --version >/dev/null 2>&1; then \
		echo "cargo-vet is not installed. You can install it using 'cargo install cargo-vet'"; \
		exit 1; \
	fi

.PHONY: check-osv-scanner-installed
check-osv-scanner-installed:  #-- Verify osv-scanner is installed and version matches tools.toml
	@if ! osv-scanner --version >/dev/null 2>&1; then \
		echo "osv-scanner is not installed. See https://google.github.io/osv-scanner/installation/"; \
		exit 1; \
	fi
	@EXPECTED=$$(bash scripts/tool-version.sh osv-scanner); \
	INSTALLED=$$(osv-scanner --version 2>&1 | grep -oE '[0-9]+\.[0-9]+\.[0-9]+' | head -1); \
	if [ "$$INSTALLED" != "$$EXPECTED" ]; then \
		printf "$(YELLOW)osv-scanner version mismatch: installed %s, expected %s (from tools.toml)$(RESET)\n" "$$INSTALLED" "$$EXPECTED"; \
	fi

# Testing tool checks
.PHONY: check-nextest-installed
check-nextest-installed:  #-- Verify cargo-nextest is installed
	@if ! cargo nextest --version >/dev/null 2>&1; then \
		echo "cargo-nextest is not installed. You can install it using 'cargo install cargo-nextest'"; \
		exit 1; \
	fi

.PHONY: check-llvm-cov-installed
check-llvm-cov-installed:  #-- Verify cargo-llvm-cov is installed
	@if ! cargo llvm-cov --version >/dev/null 2>&1; then \
		echo "cargo-llvm-cov is not installed. You can install it using 'cargo install cargo-llvm-cov'"; \
		exit 1; \
	fi

# Cargo utility checks
.PHONY: check-hack-installed
check-hack-installed:  #-- Verify cargo-hack is installed
	@if ! cargo hack --version >/dev/null 2>&1; then \
		echo "cargo-hack is not installed. You can install it using 'cargo install cargo-hack'"; \
		exit 1; \
	fi

.PHONY: check-edit-installed
check-edit-installed:  #-- Verify cargo-edit is installed
	@if ! cargo upgrade --version >/dev/null 2>&1; then \
		echo "cargo-edit is not installed. You can install it using 'cargo install cargo-edit'"; \
		exit 1; \
	fi

.PHONY: check-features
check-features: check-hack-installed  #-- Verify crate feature combinations compile correctly
	cargo hack --workspace check --each-feature --all-targets

#== Rust Testing

.PHONY: cargo-test
cargo-test: export RUST_BACKTRACE=1
cargo-test: check-nextest-installed
cargo-test:  #-- Run all Rust tests (use EXTRA_FEATURES="feature1 feature2" or HYPERSYNC=true)
ifeq ($(VERBOSE),true)
	$(info $(M) Running Rust tests with verbose output...)
	cargo nextest run --workspace --lib --tests --features "$(CARGO_FEATURES)" $(FAIL_FAST_FLAG) --profile $(NEXTEST_PROFILE) --cargo-profile $(CARGO_CI_PROFILE) --verbose
else
	$(info $(M) Running Rust tests (showing summary and failures only)...)
	cargo nextest run --workspace --lib --tests --features "$(CARGO_FEATURES)" $(FAIL_FAST_FLAG) --profile $(NEXTEST_PROFILE) --cargo-profile $(CARGO_CI_PROFILE) --status-level fail --final-status-level flaky
endif

.PHONY: cargo-test-extras
cargo-test-extras:  #-- Run all Rust tests with hypersync features (convenience shortcut)
	$(MAKE) cargo-test EXTRA_FEATURES="hypersync"

# Both core and adapter targets use identical --workspace --features flags so
# cargo sees the same feature union and does not recompile between runs.
# The -E filterset selects which tests to execute.
CORE_FILTERSET := $(subst $(eval ) , + ,$(foreach crate,$(CORE_CRATES),package($(crate))))
ADAPTER_FILTERSET := $(subst $(eval ) , + ,$(foreach crate,$(ADAPTER_CRATES),package($(crate))))

.PHONY: cargo-test-core-local
cargo-test-core-local: export RUST_BACKTRACE=1
cargo-test-core-local: check-nextest-installed
cargo-test-core-local:  #-- Run Rust tests for core crates only with direct package selection (fast local compile)
ifeq ($(VERBOSE),true)
	$(info $(M) Running Rust tests for core crates with direct package selection...)
	cargo nextest run $(foreach crate,$(CORE_CRATES),-p $(crate)) --lib --tests --features "$(CARGO_FEATURES)" $(FAIL_FAST_FLAG) --profile $(NEXTEST_PROFILE) --cargo-profile $(CARGO_CI_PROFILE) --verbose
else
	$(info $(M) Running Rust tests for core crates with direct package selection (showing summary and failures only)...)
	cargo nextest run $(foreach crate,$(CORE_CRATES),-p $(crate)) --lib --tests --features "$(CARGO_FEATURES)" $(FAIL_FAST_FLAG) --profile $(NEXTEST_PROFILE) --cargo-profile $(CARGO_CI_PROFILE) --status-level fail --final-status-level flaky
endif

.PHONY: cargo-test-core
cargo-test-core: export RUST_BACKTRACE=1
cargo-test-core: check-nextest-installed
cargo-test-core:  #-- Run Rust tests for core crates only (excludes adapters)
ifeq ($(VERBOSE),true)
	$(info $(M) Running Rust tests for core crates...)
	cargo nextest run --workspace --lib --tests --features "$(CARGO_FEATURES)" -E '$(CORE_FILTERSET)' $(FAIL_FAST_FLAG) --profile $(NEXTEST_PROFILE) --cargo-profile $(CARGO_CI_PROFILE) --verbose
else
	$(info $(M) Running Rust tests for core crates (showing summary and failures only)...)
	cargo nextest run --workspace --lib --tests --features "$(CARGO_FEATURES)" -E '$(CORE_FILTERSET)' $(FAIL_FAST_FLAG) --profile $(NEXTEST_PROFILE) --cargo-profile $(CARGO_CI_PROFILE) --status-level fail --final-status-level flaky
endif

.PHONY: cargo-test-adapters
cargo-test-adapters: export RUST_BACKTRACE=1
cargo-test-adapters: check-nextest-installed
cargo-test-adapters:  #-- Run Rust tests for adapter crates only
ifeq ($(VERBOSE),true)
	$(info $(M) Running Rust tests for adapter crates...)
	cargo nextest run --workspace --lib --tests --features "$(CARGO_FEATURES)" -E '$(ADAPTER_FILTERSET)' $(FAIL_FAST_FLAG) --profile $(NEXTEST_PROFILE) --cargo-profile $(CARGO_CI_PROFILE) --verbose
else
	$(info $(M) Running Rust tests for adapter crates (showing summary and failures only)...)
	cargo nextest run --workspace --lib --tests --features "$(CARGO_FEATURES)" -E '$(ADAPTER_FILTERSET)' $(FAIL_FAST_FLAG) --profile $(NEXTEST_PROFILE) --cargo-profile $(CARGO_CI_PROFILE) --status-level fail --final-status-level flaky
endif

# DST simulation smoke test. Compiles the in-scope crates under cfg(madsim)
# and runs every test that is sim-compatible today: all of nautilus-common,
# nautilus-network, and nautilus-execution (transport-bound tests are gated
# out at the source), plus the cross-crate seam pinning tests in nautilus-core.
# Each leg runs with the standard fixed-precision build first, then again
# under `high-precision` for the crates that consume `nautilus-model` types,
# so the seam-routed code paths are exercised under both `QuantityRaw` /
# `PriceRaw` widths (u64 vs u128). See docs/concepts/dst.md for the full
# DST scope.
.PHONY: cargo-test-sim
cargo-test-sim: export RUST_BACKTRACE=1
cargo-test-sim: export RUSTFLAGS=--cfg madsim
cargo-test-sim: check-nextest-installed
cargo-test-sim:  #-- Run DST simulation smoke tests (cfg madsim + simulation feature)
	$(info $(M) Building in-scope crates under simulation (compile gate)...)
	cargo build -p nautilus-common -p nautilus-core -p nautilus-network -p nautilus-execution --tests --lib --features simulation
	$(info $(M) Running nautilus-common tests under simulation...)
	cargo nextest run -p nautilus-common --features simulation $(FAIL_FAST_FLAG) --profile $(NEXTEST_PROFILE) --cargo-profile $(CARGO_CI_PROFILE) --status-level fail --final-status-level flaky
	$(info $(M) Running nautilus-common tests under simulation + high-precision...)
	cargo nextest run -p nautilus-common --features "simulation,high-precision" $(FAIL_FAST_FLAG) --profile $(NEXTEST_PROFILE) --cargo-profile $(CARGO_CI_PROFILE) --status-level fail --final-status-level flaky
	$(info $(M) Running nautilus-network tests under simulation...)
	cargo nextest run -p nautilus-network --features simulation $(FAIL_FAST_FLAG) --profile $(NEXTEST_PROFILE) --cargo-profile $(CARGO_CI_PROFILE) --status-level fail --final-status-level flaky
	$(info $(M) Running nautilus-execution tests under simulation...)
	cargo nextest run -p nautilus-execution --features simulation $(FAIL_FAST_FLAG) --profile $(NEXTEST_PROFILE) --cargo-profile $(CARGO_CI_PROFILE) --status-level fail --final-status-level flaky
	$(info $(M) Running nautilus-execution tests under simulation + high-precision...)
	cargo nextest run -p nautilus-execution --features "simulation,high-precision" $(FAIL_FAST_FLAG) --profile $(NEXTEST_PROFILE) --cargo-profile $(CARGO_CI_PROFILE) --status-level fail --final-status-level flaky
	$(info $(M) Running nautilus-core DST seam pinning tests under simulation...)
	cargo nextest run -p nautilus-core --features simulation -E 'test(~virtual_time)' $(FAIL_FAST_FLAG) --profile $(NEXTEST_PROFILE) --cargo-profile $(CARGO_CI_PROFILE) --status-level fail --final-status-level flaky

PLUGIN_CDYLIB_SMOKE_LIVE_FILTER := \
    test(=loader_loads_example_cdylib) \
    | test(=custom_data_registration_round_trips_via_registry) \
    | test(=live_node_loads_configured_plugin_actor_strategy_and_custom_data) \
    | test(=live_node_start_invokes_configured_plugin_actor) \
    | (test(~cdylib_actor_) & test(~normalizes_identifiers_for_plugin)) \
    | (test(~cdylib_strategy_) & test(~normalizes_identifiers))

.PHONY: cargo-test-plugin-cdylib-smoke
cargo-test-plugin-cdylib-smoke: export RUST_BACKTRACE=1
cargo-test-plugin-cdylib-smoke: check-nextest-installed
cargo-test-plugin-cdylib-smoke:  #-- Run Linux plug-in cdylib smoke tests
	@if [ "$$(uname -s)" != "Linux" ]; then \
		echo "cargo-test-plugin-cdylib-smoke requires Linux"; \
		exit 1; \
	fi
	$(info $(M) Running nautilus-plugin loader cdylib smoke test...)
	cargo nextest run \
		-p nautilus-plugin \
		--features host \
		--test load_example_cdylib \
		--run-ignored only \
		-E 'test(=loads_example_cdylib_and_walks_manifest)' \
		$(FAIL_FAST_FLAG) \
		--profile $(NEXTEST_PROFILE) \
		--cargo-profile $(CARGO_CI_PROFILE) \
		--test-threads 1 \
		--status-level fail \
		--final-status-level flaky
	$(info $(M) Running nautilus-live plug-in cdylib smoke tests...)
	cargo nextest run \
		-p nautilus-live \
		--features plugin \
		--test plugin \
		-E '$(PLUGIN_CDYLIB_SMOKE_LIVE_FILTER)' \
		$(FAIL_FAST_FLAG) \
		--profile $(NEXTEST_PROFILE) \
		--cargo-profile $(CARGO_CI_PROFILE) \
		--test-threads 1 \
		--status-level fail \
		--final-status-level flaky

.PHONY: cargo-test-core-debug
cargo-test-core-debug: export RUST_BACKTRACE=1
cargo-test-core-debug: check-nextest-installed
cargo-test-core-debug:  #-- Run Rust tests for core crates (debug profile)
	cargo nextest run --workspace --lib --tests --features "$(CARGO_FEATURES)" -E '$(CORE_FILTERSET)' $(FAIL_FAST_FLAG) --profile $(NEXTEST_PROFILE)

.PHONY: cargo-test-core-local-debug
cargo-test-core-local-debug: export RUST_BACKTRACE=1
cargo-test-core-local-debug: check-nextest-installed
cargo-test-core-local-debug:  #-- Run Rust tests for core crates with direct package selection (debug profile)
	cargo nextest run $(foreach crate,$(CORE_CRATES),-p $(crate)) --lib --tests --features "$(CARGO_FEATURES)" $(FAIL_FAST_FLAG) --profile $(NEXTEST_PROFILE)

.PHONY: cargo-test-lib
cargo-test-lib: export RUST_BACKTRACE=1
cargo-test-lib: check-nextest-installed
cargo-test-lib:  #-- Run Rust library tests only with high precision
	cargo nextest run --lib --workspace --no-default-features --features "ffi,high-precision,streaming,defi,stubs" $(FAIL_FAST_FLAG) --profile $(NEXTEST_PROFILE) --cargo-profile $(CARGO_CI_PROFILE)

.PHONY: cargo-test-standard-precision
cargo-test-standard-precision: export RUST_BACKTRACE=1
cargo-test-standard-precision: check-nextest-installed
cargo-test-standard-precision:  #-- Run Rust tests with standard precision (debug profile)
	cargo nextest run --workspace --lib --tests --features "ffi" $(FAIL_FAST_FLAG) --profile $(NEXTEST_PROFILE)

.PHONY: cargo-test-debug
cargo-test-debug: export RUST_BACKTRACE=1
cargo-test-debug: check-nextest-installed
cargo-test-debug:  #-- Run Rust tests with high precision (debug profile)
	cargo nextest run --workspace --lib --tests --features "ffi,high-precision,streaming,defi" $(FAIL_FAST_FLAG) --profile $(NEXTEST_PROFILE)

.PHONY: cargo-test-coverage
cargo-test-coverage: check-nextest-installed check-llvm-cov-installed
cargo-test-coverage:  #-- Run Rust tests with coverage reporting
	cargo llvm-cov nextest run --workspace --lib --tests --features "$(CARGO_FEATURES)"

# -----------------------------------------------------------------------------
# Library tests for a single crate
# -----------------------------------------------------------------------------
# Invoke as:
#   make cargo-test-crate-<crate_name>
# Examples:
#   make cargo-test-crate-nautilus-model
#   make cargo-test-crate-nautilus-live
#
# Enables all crate features except default. Feature list is resolved by
# crate-test-features.sh.
# -----------------------------------------------------------------------------

.PHONY: cargo-test-crate-%
cargo-test-crate-%: export RUST_BACKTRACE=1
cargo-test-crate-%: check-nextest-installed
cargo-test-crate-%:  #-- Run Rust tests for a specific crate (usage: make cargo-test-crate-<crate_name>)
	cargo nextest run --lib $(FAIL_FAST_FLAG) --profile $(NEXTEST_PROFILE) --cargo-profile $(CARGO_CI_PROFILE) -p $* --features "$$(./scripts/crate-test-features.sh $*)"

.PHONY: cargo-test-coverage-crate-%
cargo-test-coverage-crate-%: export RUST_BACKTRACE=1
cargo-test-coverage-crate-%: check-nextest-installed check-llvm-cov-installed
cargo-test-coverage-crate-%:  #-- Run Rust tests with coverage reporting for a specific crate (usage: make cargo-test-coverage-crate-<crate_name>)
	cargo llvm-cov nextest --lib $(FAIL_FAST_FLAG) --cargo-profile nextest -p $* $(if $(FEATURES),--features "$(FEATURES)")

.PHONY: cargo-test-coverage-html
cargo-test-coverage-html: check-nextest-installed check-llvm-cov-installed
cargo-test-coverage-html:  #-- Run Rust tests with HTML coverage report (opens in browser)
	cargo llvm-cov nextest --workspace --lib --tests --features "$(CARGO_FEATURES)" --html --open

.PHONY: cargo-test-coverage-crate-html-%
cargo-test-coverage-crate-html-%: export RUST_BACKTRACE=1
cargo-test-coverage-crate-html-%: check-nextest-installed check-llvm-cov-installed
cargo-test-coverage-crate-html-%:  #-- Run coverage for specific crate with HTML report (usage: make cargo-test-coverage-crate-html-<crate_name>)
	cargo llvm-cov nextest --lib $(FAIL_FAST_FLAG) --cargo-profile nextest -p $* $(if $(FEATURES),--features "$(FEATURES)") --html --open

# -----------------------------------------------------------------------------
# Miri (UB detection)
# -----------------------------------------------------------------------------
# Runs library and selected integration tests under Miri to detect undefined
# behaviour: invalid pointer operations, aliasing violations (Stacked/Tree
# Borrows), uninitialised reads, and unsound `unsafe` impls. Requires a nightly
# toolchain with the `miri` component installed.
#
# Features: `ffi` and `defi` are intentionally disabled. Miri cannot execute
# most foreign FFI, and `defi` pulls in `alloy-primitives`, which is out of scope here. The
# `--lib` filter keeps doctests out of the run as well.
#
# Proptest cases are dialled down via `PROPTEST_CASES` since Miri is roughly
# 10-100x slower than native execution. `MIRIFLAGS` enables disable-isolation
# so tests that read environment variables (e.g. PATH probes) work. Most runs
# use strict provenance; the collections slice uses permissive provenance to
# match arc-swap's Miri policy.
# -----------------------------------------------------------------------------

# Override these on the command line if needed, e.g.:
#   make cargo-miri-core MIRI_TOOLCHAIN=nightly-2026-04-16
#   make cargo-miri-core MIRI_CORE_FILTER=...
#   make cargo-miri-core MIRI_CORE_ARC_SWAP_FILTER=...
#   make cargo-miri-plugin MIRI_PLUGIN_FILTER=...
#   make cargo-miri-plugin MIRI_PLUGIN_MANIFEST_FILTER=...
MIRI_TOOLCHAIN ?= nightly
MIRI_FLAGS ?= -Zmiri-disable-isolation -Zmiri-strict-provenance
MIRI_CORE_ARC_SWAP_FLAGS ?= -Zmiri-disable-isolation -Zmiri-permissive-provenance
MIRI_PLUGIN_MANIFEST_FLAGS ?= $(MIRI_FLAGS) -Zmiri-ignore-leaks
MIRI_PROPTEST_CASES ?= 4

# Default test filters target modules with `unsafe` blocks or hand-rolled
# pointer/integer code where Miri provides the most signal. Miri runs ~10-100x
# slower than native, so we narrow the default scope; pass the override above
# (or `MIRI_CORE_FILTER=`) to widen it.
MIRI_CORE_FILTER ?= -E 'test(/^(string::stack_str|nanos|uuid|hex|correctness|datetime)::/)'
# `collections::` covers AtomicMap/AtomicSet, which are backed by arc-swap.
# arc-swap runs Miri with permissive provenance, so use the same provenance
# policy for this slice while keeping strict provenance for in-tree pointer code.
MIRI_CORE_ARC_SWAP_FILTER ?= -E 'test(/^collections::/)'
# `test_price_to_order_id_{comprehensive_collision_check,realistic_orderbook_prices}`
# iterate over the full price space to verify hash uniqueness. They run for
# multiple hours under the Miri interpreter and exercise no unsafe, so we skip
# them here while keeping the rest of `orderbook::` in scope.
MIRI_MODEL_FILTER ?= -E 'test(/^(types::|identifiers::|orderbook::)/) and not test(=orderbook::aggregation::tests::test_price_to_order_id_comprehensive_collision_check) and not test(=orderbook::aggregation::tests::test_price_to_order_id_realistic_orderbook_prices)'
# Keep the plug-in Miri lane focused on the ABI boundary, raw handle ownership,
# panic guards, and command handles. Manifest fixtures model static cdylib
# storage with `Box::leak`, so that slice runs with leak detection disabled
# while the ownership-focused tests stay strict. `custom_data_dispatch` covers
# the integration path for clone/drop/equality and decoded handle arrays
# without enabling the host feature or dynamic loading.
MIRI_PLUGIN_FILTER ?= -E 'test(/^(boundary|host|panic|surfaces::commands)::/)'
MIRI_PLUGIN_MANIFEST_FILTER ?= -E 'test(/^manifest::/)'

.PHONY: check-miri-installed
check-miri-installed:
	@if ! cargo +$(MIRI_TOOLCHAIN) miri --version >/dev/null 2>&1; then \
		echo "cargo-miri is not installed for toolchain $(MIRI_TOOLCHAIN)"; \
		echo "Install with: rustup toolchain install $(MIRI_TOOLCHAIN) --component miri"; \
		exit 1; \
	fi

.PHONY: cargo-miri-core
cargo-miri-core: export RUST_BACKTRACE=1
cargo-miri-core: export PROPTEST_CASES=$(MIRI_PROPTEST_CASES)
cargo-miri-core: check-miri-installed check-nextest-installed
cargo-miri-core:  #-- Run nautilus-core library tests under Miri to detect UB
	$(info $(M) Running nautilus-core tests under Miri with strict provenance (filter: $(MIRI_CORE_FILTER))...)
	MIRIFLAGS="$(MIRI_FLAGS)" cargo +$(MIRI_TOOLCHAIN) miri nextest run -p nautilus-core --no-default-features --lib $(MIRI_CORE_FILTER)
	$(info $(M) Running nautilus-core collections tests under Miri with permissive provenance (filter: $(MIRI_CORE_ARC_SWAP_FILTER))...)
	MIRIFLAGS="$(MIRI_CORE_ARC_SWAP_FLAGS)" cargo +$(MIRI_TOOLCHAIN) miri nextest run -p nautilus-core --no-default-features --lib $(MIRI_CORE_ARC_SWAP_FILTER)

.PHONY: cargo-miri-model
cargo-miri-model: export RUST_BACKTRACE=1
cargo-miri-model: export MIRIFLAGS=$(MIRI_FLAGS)
cargo-miri-model: export PROPTEST_CASES=$(MIRI_PROPTEST_CASES)
cargo-miri-model: check-miri-installed check-nextest-installed
cargo-miri-model:  #-- Run nautilus-model library tests under Miri to detect UB
	$(info $(M) Running nautilus-model tests under Miri (filter: $(MIRI_MODEL_FILTER))...)
	cargo +$(MIRI_TOOLCHAIN) miri nextest run -p nautilus-model --no-default-features --lib $(MIRI_MODEL_FILTER)

.PHONY: cargo-miri-plugin
cargo-miri-plugin: export RUST_BACKTRACE=1
cargo-miri-plugin: export PROPTEST_CASES=$(MIRI_PROPTEST_CASES)
cargo-miri-plugin: check-miri-installed check-nextest-installed
cargo-miri-plugin:  #-- Run nautilus-plugin boundary tests under Miri to detect UB
	$(info $(M) Running nautilus-plugin library tests under Miri (filter: $(MIRI_PLUGIN_FILTER))...)
	MIRIFLAGS="$(MIRI_FLAGS)" \
		cargo +$(MIRI_TOOLCHAIN) miri nextest run \
		-p nautilus-plugin \
		--no-default-features \
		--lib \
		$(MIRI_PLUGIN_FILTER)
	$(info $(M) Running nautilus-plugin manifest tests under Miri (filter: $(MIRI_PLUGIN_MANIFEST_FILTER))...)
	MIRIFLAGS="$(MIRI_PLUGIN_MANIFEST_FLAGS)" \
		cargo +$(MIRI_TOOLCHAIN) miri nextest run \
		-p nautilus-plugin \
		--no-default-features \
		--lib \
		$(MIRI_PLUGIN_MANIFEST_FILTER)
	$(info $(M) Running nautilus-plugin custom data dispatch tests under Miri...)
	MIRIFLAGS="$(MIRI_FLAGS)" \
		cargo +$(MIRI_TOOLCHAIN) miri nextest run \
		-p nautilus-plugin \
		--no-default-features \
		--test custom_data_dispatch

.PHONY: cargo-miri
cargo-miri:  #-- Run Miri across the in-scope foundational and plug-in crates
	$(MAKE) cargo-miri-core
	$(MAKE) cargo-miri-model
	$(MAKE) cargo-miri-plugin

#------------------------------------------------------------------------------
# Benchmarks
#------------------------------------------------------------------------------

# Local batch selection retained until BPO-002 materializes the hosted workflow
CI_BENCH_CRATES := nautilus-core nautilus-model nautilus-common nautilus-live

# NOTE:
# - We invoke `cargo bench` *once per crate* to avoid the well-known
#   "mixed panic strategy" linker error that appears when crates which specify
#   different `panic` strategies (e.g. `abort` for cdylib/staticlib targets vs
#   `unwind` for Criterion) are linked into the *same* benchmark binary.
# - Cargo will still reuse compiled artifacts between iterations, so the cost
#   of the extra invocations is marginal while the linker remains happy.

.PHONY: cargo-ci-benches
cargo-ci-benches:  #-- Run the local Rust benchmark batch selection
	@for crate in $(CI_BENCH_CRATES); do \
	  echo "Running benches for $$crate"; \
	  cargo bench -p $$crate --profile bench --benches --no-fail-fast; \
	done

.PHONY: init-services
init-services:  #-- Initialize development services eg. for integration tests (start containers and setup database)
	$(info $(M) Initializing development services...)
	@$(MAKE) start-services
	@printf "$(PURPLE)Waiting for PostgreSQL to be ready...$(RESET)\n"
	@sleep 10
	@$(MAKE) init-db

.PHONY: start-services
start-services:  #-- Start development services (without reinitializing database)
	$(info $(M) Starting development services...)
	docker compose -f .docker/docker-compose.yml up -d

.PHONY: stop-services
stop-services:  #-- Stop development services (preserves data)
	$(info $(M) Stopping development services...)
	docker compose -f .docker/docker-compose.yml down

.PHONY: purge-services
purge-services:  #-- Purge all development services (stop containers and remove volumes)
	$(info $(M) Purging integration test services...)
	docker compose -f .docker/docker-compose.yml down -v

.PHONY: init-db
init-db:  #-- Initialize PostgreSQL database schema
	$(info $(M) Initializing PostgreSQL database schema...)
	cat schema/sql/types.sql schema/sql/tables.sql schema/sql/functions.sql schema/sql/partitions.sql | docker exec -i nautilus-database psql -U nautilus -d nautilus

#== CLI Tools

.PHONY: install-cli
install-cli:  #-- Install Nautilus CLI tool from source
	cargo install --path crates/cli --bin nautilus --locked --force

#== Internal

.PHONY: help
help:  #-- Show this help message and exit
	@printf "NautilusTrader Makefile\n\n"
	@printf "$(GRAY)Requires GNU Make. Windows users can install it via MSYS2 or WSL.$(RESET)\n\n"
	@printf "$(GREEN)Usage:$(RESET) make $(CYAN)<target>$(RESET)\n\n"
	@printf "$(GRAY)Tips: Use $(CYAN)make <target> V=1$(GRAY) for verbose output$(RESET)\n"
	@printf "$(GRAY)      Use $(CYAN)make <target> VERBOSE=false$(GRAY) to disable verbose output for build-debug and cargo-test$(RESET)\n\n"

	@printf "⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣠⣴⣶⡟⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀\n"
	@printf "⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣰⣾⣿⣿⣿⠀⢸⣿⣿⣿⣿⣶⣶⣤⣀⠀⠀⠀⠀⠀\n"
	@printf "⠀⠀⠀⠀⠀⠀⢀⣴⡇⢀⣾⣿⣿⣿⣿⣿⠀⣾⣿⣿⣿⣿⣿⣿⣿⠿⠓⠀⠀⠀⠀\n"
	@printf "⠀⠀⠀⠀⠀⣰⣿⣿⡀⢸⣿⣿⣿⣿⣿⣿⠀⣿⣿⣿⣿⣿⣿⠟⠁⣠⣄⠀⠀⠀⠀\n"
	@printf "⠀⠀⠀⠀⢠⣿⣿⣿⣇⠀⢿⣿⣿⣿⣿⣿⠀⢻⣿⣿⣿⡿⢃⣠⣾⣿⣿⣧⡀⠀⠀\n"
	@printf "⠀⠀⠀⠠⣾⣿⣿⣿⣿⣿⣧⠈⠋⢀⣴⣧⠀⣿⡏⢠⡀⢸⣿⣿⣿⣿⣿⣿⣿⡇⠀\n"
	@printf "⠀⠀⠀⣀⠙⢿⣿⣿⣿⣿⣿⠇⢠⣿⣿⣿⡄⠹⠃⠼⠃⠈⠉⠛⠛⠛⠛⠛⠻⠇⠀\n"
	@printf "⠀⠀⢸⡟⢠⣤⠉⠛⠿⢿⣿⠀⢸⣿⡿⠋⣠⣤⣄⠀⣾⣿⣿⣶⣶⣶⣦⡄⠀⠀⠀\n"
	@printf "⠀⠀⠸⠀⣾⠏⣸⣷⠂⣠⣤⠀⠘⢁⣴⣾⣿⣿⣿⡆⠘⣿⣿⣿⣿⣿⣿⠀⠀⠀⠀\n"
	@printf "⠀⠀⠀⠀⠛⠀⣿⡟⠀⢻⣿⡄⠸⣿⣿⣿⣿⣿⣿⣿⡀⠘⣿⣿⣿⣿⠟⠀⠀⠀⠀\n"
	@printf "⠀⠀⠀⠀⠀⠀⣿⠇⠀⠀⢻⡿⠀⠈⠻⣿⣿⣿⣿⣿⡇⠀⢹⣿⠿⠋⠀⠀⠀⠀⠀\n"
	@printf "⠀⠀⠀⠀⠀⠀⠋⠀⠀⠀⡘⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠈⠁⠀⠀⠀⠀⠀⠀⠀\n"

	@awk '\
	BEGIN { \
		FS = ":.*#--"; \
		target_maxlen = 0; \
		GREEN = "\033[0;32m"; \
		CYAN = "\033[0;36m"; \
		RESET = "\033[0m"; \
	} \
	/^[$$()% a-zA-Z0-9_-]+:.*?#--/ { \
		if (length($$1) > target_maxlen) target_maxlen = length($$1); \
		targets[NR] = $$1; descriptions[NR] = $$2; \
	} \
	/^#==/ { \
		groups[NR] = substr($$0, 5); \
	} \
	END { \
		for (i = 1; i <= NR; i++) { \
			if (groups[i]) { \
				printf "\n" GREEN "%s:" RESET "\n", groups[i]; \
			} else if (targets[i]) { \
				printf "  " CYAN "%-*s" RESET " %s\n", target_maxlen, targets[i], descriptions[i]; \
			} \
		} \
	}' $(MAKEFILE_LIST)
