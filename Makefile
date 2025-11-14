# Mina Makefile

# Rust
# This should be in line with the verison in:
# - Makefile
# - ./github/workflows/docs.yaml
# - ./github/workflows/fmt.yaml
# - ./github/workflows/lint.yaml
NIGHTLY_RUST_VERSION = "nightly"
NODE_VERSION := $(shell cat .nvmrc)

# WebAssembly
WASM_BINDGEN_CLI_VERSION = "0.2.99"

# TOML formatter
TAPLO_CLI_VERSION = "0.9.3"

# Docker
DOCKER_ORG ?= o1labs

# PostgreSQL configuration for archive node
OPEN_ARCHIVE_ADDRESS ?= http://localhost:3007
PG_USER ?= mina
PG_PW	?= minamina
PG_DB	?= mina_archive
PG_HOST	?= localhost
PG_PORT	?= 5432

# Block producer configuration
PRODUCER_KEY_FILENAME ?= ./mina-workdir/producer-key
COINBASE_RECEIVER ?=
MINA_LIBP2P_EXTERNAL_IP ?=
MINA_LIBP2P_PORT ?= 8302

# Utilities
NETWORK ?= devnet
VERBOSITY ?= info
GIT_COMMIT := $(shell git rev-parse --short=8 HEAD)

# Documentation server port
DOCS_PORT ?= 3000

OPAM_PATH := $(shell command -v opam 2>/dev/null)

ifdef OPAM_PATH
# This captures what `eval $(opam env)` would set in your shell
OPAM_ENV := $(shell eval $$(opam env) && env | grep '^OPAM\|^PATH\|^CAML' | sed 's/^/export /')
export $(shell eval $$(opam env) && env | grep '^OPAM\|^PATH\|^CAML' | cut -d= -f1)
$(foreach v,$(shell eval $$(opam env) && env | grep '^OPAM\|^PATH\|^CAML'),$(eval export $(v)))
endif

.PHONY: help
help: ## Ask for help!
	@echo "Mina Rust Makefile - Common Variables:"
	@echo "  DOCS_PORT=<port>     Set documentation server port (default: 3000)"
	@echo "  NETWORK=<network>    Set network (default: devnet)"
	@echo "  VERBOSITY=<level>    Set logging verbosity (default: info)"
	@echo ""
	@echo "Examples:"
	@echo "  make docs-serve DOCS_PORT=8080    # Start docs server on port 8080"
	@echo "  make run-node NETWORK=mainnet     # Run node on mainnet"
	@echo ""
	@echo "Available targets:"
	@grep -E '^[a-zA-Z0-9_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-30s\033[0m %s\n", $$1, $$2}'

.PHONY: build
build: ## Build the project in debug mode
	cargo build

.PHONY: build-ledger
build-ledger: download-circuits ## Build the ledger binary and library, requires nightly Rust
	@cd ledger && cargo +$(NIGHTLY_RUST_VERSION) build --release --tests

build-node-native: ## Build the package mina-node-native with all features and tests
	@cargo build -p mina-node-native --all-features --release --tests

.PHONY: build-release
build-release: ## Build the project in release mode
	@cargo build --release --package=cli --bin mina

.PHONY: build-testing
build-testing: ## Build the testing binary with scenario generators
	cargo build --release --features scenario-generators --bin mina-node-testing

.PHONY: build-tests
build-tests: ## Build tests for scenario testing
	@mkdir -p target/release/tests
	@cargo build --release --tests \
		--package=mina-node-testing \
		--package=cli
	@cargo build --release --tests \
		--package=mina-node-testing \
		--package=cli \
		--message-format=json > cargo-build-test.json
	@jq -r '. | select(.executable != null and (.target.kind | (contains(["test"])))) | [.target.name, .executable ] | @tsv' \
		cargo-build-test.json > tests.tsv
	@while read NAME FILE; do \
		cp -a $$FILE target/release/tests/$$NAME; \
	done < tests.tsv

.PHONY: build-tests-webrtc
build-tests-webrtc: ## Build tests for WebRTC
	@mkdir -p target/release/tests
	@cargo build --release --tests \
		--package=mina-node-testing \
		--package=cli
# Update ./.gitignore accordingly if cargo-build-test.json is changed
	@cargo build --release \
		--features=scenario-generators,p2p-webrtc \
		--package=mina-node-testing \
		--tests \
		--message-format=json \
		> cargo-build-test.json
# Update ./.gitignore accordingly if tests.json is changed
	@jq -r '. | select(.executable != null and (.target.kind | (contains(["test"])))) | [.target.name, .executable ] | @tsv' cargo-build-test.json > tests.tsv
	@while read NAME FILE; do \
		cp -a $$FILE target/release/tests/webrtc_$$NAME; \
	done < tests.tsv

.PHONY: build-vrf
build-vrf: ## Build the VRF package
	@cd vrf && cargo +$(NIGHTLY_RUST_VERSION) build --release --tests

.PHONY: build-wasm
build-wasm: ## Build WebAssembly node
	@cd node/web && cargo +${NIGHTLY_RUST_VERSION} build \
		--release --target wasm32-unknown-unknown
# Update ./.gitignore accordingly if the out-dir is changed
	@wasm-bindgen --keep-debug --web \
		--out-dir pkg \
		target/wasm32-unknown-unknown/release/mina_node_web.wasm

.PHONY: build-benches
build-benches: ## Build all benchmarks without running them
	@cargo bench --no-run

.PHONY: build-bench-database
build-bench-database: ## Build ledger database benchmark
	@cargo bench --bench database --no-run

.PHONY: bench
bench: ## Run all benchmarks
	@cargo bench

.PHONY: bench-database
bench-database: ## Run ledger database benchmark
	@cargo bench --bench database

.PHONY: check
check: ## Check code for compilation errors
	cargo check --all-targets

.PHONY: check-tx-fuzzing
check-tx-fuzzing: ## Check the transaction fuzzing tools, requires nightly Rust
	@cd tools/fuzzing && cargo +$(NIGHTLY_RUST_VERSION) check

.PHONY: check-format
check-format: ## Check code formatting
	cargo +$(NIGHTLY_RUST_VERSION) fmt -- --check
	taplo format --check

.PHONY: check-md
check-md: ## Check if markdown and MDX files are properly formatted
	@echo "Checking markdown and MDX formatting..."
	npx prettier --check "**/*.md" "**/*.mdx"
	@echo "Markdown and MDX format check completed."

.PHONY: fix-trailing-whitespace
fix-trailing-whitespace: ## Remove trailing whitespaces from all files
	@echo "Removing trailing whitespaces from all files..."
	@find . -type f \( \
		-name "*.rs" -o -name "*.toml" -o -name "*.md" -o -name "*.yaml" \
		-o -name "*.yml" -o -name "*.ts" -o -name "*.tsx" \
		-o -name "*.js" -o -name "*.jsx" -o -name "*.sh" \) \
		-not -path "./target/*" \
		-not -path "./node_modules/*" \
		-not -path "./frontend/node_modules/*" \
		-not -path "./frontend/dist/*" \
		-not -path "./website/node_modules/*" \
		-not -path "./website/build/*" \
		-not -path "./website/static/api-docs/*" \
		-not -path "./website/.docusaurus/*" \
		-not -path "./.git/*" \
		-exec sh -c 'echo "Processing: $$1"; sed -i"" "s/[[:space:]]*$$//" "$$1"' _ {} \; && \
		echo "Trailing whitespaces removed."

.PHONY: check-trailing-whitespace
check-trailing-whitespace: ## Check for trailing whitespaces in source files
	@echo "Checking for trailing whitespaces..."
	@files_with_trailing_ws=$$(find . -type f \( \
		-name "*.rs" -o -name "*.toml" -o -name "*.md" -o -name "*.yaml" \
		-o -name "*.yml" -o -name "*.ts" -o -name "*.tsx" \
		-o -name "*.js" -o -name "*.jsx" -o -name "*.sh" \) \
		-not -path "./target/*" \
		-not -path "./node_modules/*" \
		-not -path "./frontend/node_modules/*" \
		-not -path "./frontend/dist/*" \
		-not -path "./website/node_modules/*" \
		-not -path "./website/build/*" \
		-not -path "./website/static/api-docs/*" \
		-not -path "./website/.docusaurus/*" \
		-not -path "./.git/*" \
		-exec grep -l '[[:space:]]$$' {} + 2>/dev/null || true); \
	if [ -n "$$files_with_trailing_ws" ]; then \
		echo "❌ Files with trailing whitespaces found:"; \
		echo "$$files_with_trailing_ws" | sed 's/^/  /'; \
		echo ""; \
		echo "Run 'make fix-trailing-whitespace' to fix automatically."; \
		exit 1; \
	else \
		echo "✅ No trailing whitespaces found."; \
	fi

.PHONY: clean
clean: ## Clean build artifacts
	cargo clean

.PHONY: download-circuits
download-circuits: ## Download the circuits used by Mina from GitHub
	@if [ ! -d "circuit-blobs" ]; then \
	  git clone --depth 1 https://github.com/o1-labs/circuit-blobs.git -b dw/add-berkeley-687bf44e97328e1cc0e85291663009410f64bd99; \
	  ln -s "$$PWD"/circuit-blobs/3.0.0mainnet ledger/; \
	  ln -s "$$PWD"/circuit-blobs/berkeley-devnet ledger/; \
	else \
	  echo "circuit-blobs already exists, skipping download."; \
	fi

.PHONY: format
format: ## Format code using rustfmt and taplo
	cargo +$(NIGHTLY_RUST_VERSION) fmt
	taplo format

.PHONY: format-md
format-md: ## Format all markdown and MDX files to wrap at 80 characters
	@echo "Formatting markdown and MDX files..."
	npx prettier --write "**/*.md" "**/*.mdx"
	@echo "Markdown and MDX files have been formatted to 80 characters."

.PHONY: lint
lint: ## Run linter (clippy)
	cargo clippy --all-targets -- -D warnings --allow clippy::mutable_key_type

.PHONY: lint-bash
lint-bash: ## Check all shell scripts using shellcheck
	@echo "Running shellcheck on shell scripts..."
	@find . -name "*.sh" \
		-not -path "*/target/*" \
		-not -path "*/node_modules/*" \
		-not -path "*/website/docs/developers/scripts/setup/*" \
		-not -path "*/website/docs/developers/scripts/frontend/*" \
		-print0 | xargs -0 shellcheck
	@echo "Shellcheck completed successfully!"

.PHONY: lint-dockerfiles
lint-dockerfiles: ## Check all Dockerfiles using hadolint
	@if [ "$$GITHUB_ACTIONS" = "true" ]; then \
		OUTPUT=$$(find . -name "Dockerfile*" -type f -not -path "*/node_modules/*" -exec hadolint {} \;); \
		if [ -n "$$OUTPUT" ]; then \
			echo "$$OUTPUT"; \
			exit 1; \
		fi; \
	else \
		OUTPUT=$$(find . -name "Dockerfile*" -type f -not -path "*/node_modules/*" -exec sh -c 'docker run --rm -i hadolint/hadolint < "$$1"' _ {} \;); \
		if [ -n "$$OUTPUT" ]; then \
			echo "$$OUTPUT"; \
			exit 1; \
		fi; \
	fi

.PHONY: setup-wasm
setup-wasm: ## Setup the WebAssembly toolchain, using nightly
		@ARCH=$$(uname -m); \
		OS=$$(uname -s | tr A-Z a-z); \
		case $$OS in \
			linux) OS_PART="unknown-linux-gnu" ;; \
			darwin) OS_PART="apple-darwin" ;; \
			*) echo "Unsupported OS: $$OS" && exit 1 ;; \
		esac; \
		case $$ARCH in \
			x86_64) ARCH_PART="x86_64" ;; \
			aarch64) ARCH_PART="aarch64" ;; \
			arm64) ARCH_PART="aarch64" ;; \
			*) echo "Unsupported architecture: $$ARCH" && exit 1 ;; \
		esac; \
		TARGET="$$ARCH_PART-$$OS_PART"; \
		echo "Installing nightly toolchain: ${NIGHTLY_RUST_VERSION}-$$TARGET"; \
		rustup toolchain install ${NIGHTLY_RUST_VERSION}-$$TARGET; \
		echo "Installing components for ${NIGHTLY_RUST_VERSION}-$$TARGET with wasm32 target"; \
		rustup component add rust-src --toolchain ${NIGHTLY_RUST_VERSION}-$$TARGET; \
		rustup component add rustfmt --toolchain ${NIGHTLY_RUST_VERSION}-$$TARGET; \
		rustup target add wasm32-unknown-unknown --toolchain ${NIGHTLY_RUST_VERSION}-$$TARGET; \
		cargo install wasm-bindgen-cli --version ${WASM_BINDGEN_CLI_VERSION}

.PHONY: setup-taplo
setup-taplo: ## Install taplo TOML formatter
	@if taplo --version 2>/dev/null | grep -q ${TAPLO_CLI_VERSION}; then \
		echo "taplo ${TAPLO_CLI_VERSION} already installed"; \
	else \
		cargo +nightly install taplo-cli --version ${TAPLO_CLI_VERSION} --force; \
	fi

.PHONY: setup
setup: setup-taplo setup-wasm ## Setup development environment

.PHONY: test
test: ## Run tests
	cargo test

.PHONY: test-ledger
test-ledger: build-ledger ## Run ledger tests in release mode, requires nightly Rust
	@cd ledger && cargo +$(NIGHTLY_RUST_VERSION) test --release -- -Z unstable-options --report-time

.PHONY: test-p2p
test-p2p: ## Run P2P tests
	cargo test -p p2p --tests --release

.PHONY: test-release
test-release: ## Run tests in release mode
	cargo test --release

.PHONY: test-vrf
test-vrf: ## Run VRF tests, requires nightly Rust
	@cd vrf && cargo +$(NIGHTLY_RUST_VERSION) test --release -- \
		-Z unstable-options --report-time

.PHONY: test-account
test-account: ## Run account tests
	@cargo test -p mina-node-account

.PHONY: test-wallet
test-wallet: ## Run wallet CLI end-to-end tests
	@echo "Running wallet CLI tests..."
	@for script in .github/scripts/wallet/*.sh; do \
		echo "Running $$script..."; \
		$$script || exit 1; \
	done
	@echo "All wallet tests passed!"

.PHONY: test-p2p-messages
test-p2p-messages:
	cargo test -p mina-p2p-messages --tests --release

.PHONY: test-node-native
test-node-native: build-node-native ## Run the unit/integration tests of the package mina-node-native
	cargo test -p mina-node-native --all-features --release --tests

.PHONY: nextest
nextest: ## Run tests with cargo-nextest for faster execution
	@cargo nextest run

.PHONY: nextest-release
nextest-release: ## Run tests in release mode with cargo-nextest
	@cargo nextest run --release

.PHONY: nextest-p2p
nextest-p2p: ## Run P2P tests with cargo-nextest
	@cargo nextest run -p p2p --tests

.PHONY: nextest-ledger
nextest-ledger: build-ledger ## Run ledger tests with cargo-nextest, requires nightly Rust
	@cd ledger && cargo +$(NIGHTLY_RUST_VERSION) nextest run --release

.PHONY: nextest-vrf
nextest-vrf: ## Run VRF tests with cargo-nextest, requires nightly Rust
	@cd vrf && cargo +$(NIGHTLY_RUST_VERSION) nextest run --release

# Docker build targets

.PHONY: docker-build-all
docker-build-all: docker-build-bootstrap-sandbox docker-build-debugger \
	docker-build-frontend docker-build-fuzzing \
	docker-build-light docker-build-light-focal docker-build-mina \
	docker-build-mina-testing \
	docker-build-test ## Build all Docker images

.PHONY: docker-build-bootstrap-sandbox
docker-build-bootstrap-sandbox: ## Build bootstrap sandbox Docker image
	docker build -t $(DOCKER_ORG)/mina-rust-bootstrap-sandbox:$(GIT_COMMIT) \
		tools/bootstrap-sandbox/

.PHONY: docker-build-debugger
docker-build-debugger: ## Build debugger Docker image
	docker build -t $(DOCKER_ORG)/mina-rust-debugger:$(GIT_COMMIT) \
		-f node/testing/docker/Dockerfile.debugger node/testing/docker/

.PHONY: docker-build-frontend
docker-build-frontend: ## Build frontend Docker image
	@echo "Generating .env.docker file..."
	@bash ./frontend/docker/generate-docker-env.sh
	@ARCH=$$(uname -m); \
	case $$ARCH in \
		x86_64) PLATFORM="linux/amd64" ;; \
		aarch64|arm64) PLATFORM="linux/arm64" ;; \
		*) echo "Unsupported architecture: $$ARCH" && exit 1 ;; \
	esac; \
	echo "Building for platform: $$PLATFORM"; \
	docker buildx build \
		--build-arg NODE_VERSION=$(NODE_VERSION) \
		--platform $$PLATFORM \
		--tag $(DOCKER_ORG)/mina-rust-frontend:$(GIT_COMMIT) \
		--file ./frontend/Dockerfile \
		./

.PHONY: docker-build-fuzzing
docker-build-fuzzing: ## Build fuzzing Docker image
	docker build -t $(DOCKER_ORG)/mina-rust-fuzzing:$(GIT_COMMIT) tools/fuzzing/

.PHONY: docker-build-light
docker-build-light: ## Build light Docker image
	docker build -t $(DOCKER_ORG)/mina-rust-light:$(GIT_COMMIT) \
		-f node/testing/docker/Dockerfile.light node/testing/docker/

.PHONY: docker-build-light-focal
docker-build-light-focal: ## Build light focal Docker image
	docker build -t $(DOCKER_ORG)/mina-rust-light-focal:$(GIT_COMMIT) \
		-f node/testing/docker/Dockerfile.light.focal node/testing/docker/

.PHONY: docker-build-mina
docker-build-mina: ## Build main Mina Docker image
	@ARCH=$$(uname -m); \
	case $$ARCH in \
		x86_64) PLATFORM="linux/amd64" ;; \
		aarch64|arm64) PLATFORM="linux/arm64" ;; \
		*) echo "Unsupported architecture: $$ARCH" && exit 1 ;; \
	esac; \
	echo "Building for platform: $$PLATFORM"; \
	docker buildx build \
		--platform $$PLATFORM \
		--tag $(DOCKER_ORG)/mina-rust:$(GIT_COMMIT) \
		.

.PHONY: docker-build-mina-testing
docker-build-mina-testing: ## Build Mina testing Docker image
	docker build -t $(DOCKER_ORG)/mina-rust-testing:$(GIT_COMMIT) \
		-f node/testing/docker/Dockerfile.mina node/testing/docker/

.PHONY: docker-build-test
docker-build-test: ## Build test Docker image
	docker build -t $(DOCKER_ORG)/mina-rust-test:$(GIT_COMMIT) \
		-f node/testing/docker/Dockerfile.test node/testing/docker/

# Docker push targets

.PHONY: docker-push-mina
docker-push-mina: ## Push main Mina Docker image to DockerHub
	@docker push $(DOCKER_ORG)/mina-rust:$(GIT_COMMIT)

.PHONY: docker-push-frontend
docker-push-frontend: ## Push frontend Docker image to DockerHub
	@docker push $(DOCKER_ORG)/mina-rust-frontend:$(GIT_COMMIT)

# Node running targets
.PHONY: run-node
run-node: build-release ## Run a basic node (NETWORK=devnet, VERBOSITY=info)
	@cargo run --release --package=cli --bin mina -- node --network $(NETWORK) --verbosity $(VERBOSITY)

# Postgres related targets + archive node
.PHONY: run-archive
run-archive: build-release ## Run an archive node with local storage
	MINA_ARCHIVE_ADDRESS=$(MINA_ARCHIVE_ADDRESS) \
		cargo run --bin mina \
		--release -- \
		node \
		--archive-archiver-process \
		--archive-local-storage \
		--network $(NETWORK)

.PHONY: run-block-producer
run-block-producer: build-release ## Run a block producer node on $(NETWORK) network
	@if [ ! -f "$(PRODUCER_KEY_FILENAME)" ]; then \
		echo "Error: Producer key not found at $(PRODUCER_KEY_FILENAME)"; \
		echo "Please place your producer private key at $(PRODUCER_KEY_FILENAME)"; \
		exit 1; \
	fi
	cargo run \
		--bin mina \
		--package=cli \
		--release -- \
		node \
		--producer-key $(PRODUCER_KEY_FILENAME) \
		$(if $(COINBASE_RECEIVER),--coinbase-receiver $(COINBASE_RECEIVER)) \
		$(if $(MINA_LIBP2P_EXTERNAL_IP),--libp2p-external-ip $(MINA_LIBP2P_EXTERNAL_IP)) \
		$(if $(MINA_LIBP2P_PORT),--libp2p-port $(MINA_LIBP2P_PORT)) \
		--network $(NETWORK)


.PHONY: generate-block-producer-key
generate-block-producer-key: build-release ## Generate a new block producer key pair (fails if keys exist, use PRODUCER_KEY_FILENAME to customize, MINA_PRIVKEY_PASS for password)
	@if [ -f "$(PRODUCER_KEY_FILENAME)" ] || [ -f "$(PRODUCER_KEY_FILENAME).pub" ]; then \
		echo "Error: Producer key already exists at $(PRODUCER_KEY_FILENAME) or public key exists at $(PRODUCER_KEY_FILENAME).pub"; \
		echo ""; \
		echo "To generate a key with a different filename, set PRODUCER_KEY_FILENAME:"; \
		echo "  make generate-block-producer-key PRODUCER_KEY_FILENAME=./path/to/new-key"; \
		echo ""; \
		echo "Or remove the existing key first to regenerate it."; \
		exit 1; \
	fi
	@mkdir -p mina-workdir
	@echo "Generating new encrypted block producer key..."
	@OUTPUT=$$($(if $(MINA_PRIVKEY_PASS),MINA_PRIVKEY_PASS="$(MINA_PRIVKEY_PASS)") cargo run --release --package=cli --bin mina -- misc mina-encrypted-key --file $(PRODUCER_KEY_FILENAME)); \
	PUBLIC_KEY=$$(echo "$$OUTPUT" | grep "public key:" | cut -d' ' -f3); \
	chmod 600 $(PRODUCER_KEY_FILENAME); \
	echo ""; \
	echo "✓ Generated new encrypted producer key:"; \
	echo "  Encrypted key saved to: $(PRODUCER_KEY_FILENAME)"; \
	echo "  Public key: $$PUBLIC_KEY, saved to $(PRODUCER_KEY_FILENAME).pub"; \
	echo ""; \
	echo "⚠️  IMPORTANT: Keep your encrypted key file and password secure and backed up!"

.PHONY: postgres-clean
postgres-clean:
	@echo "Dropping DB: ${PG_DB} and user: ${PG_USER}"
	@sudo -u postgres psql -c "DROP DATABASE IF EXISTS ${PG_DB}"
	@sudo -u postgres psql -c "DROP DATABASE IF EXISTS ${PG_USER}"
	@sudo -u postgres psql -c "DROP ROLE IF EXISTS ${PG_USER}"
	@echo "Cleanup complete."

.PHONY: postgres-setup
postgres-setup: ## Set up PostgreSQL database for archive node
	@echo "Setting up PostgreSQL database: ${PG_DB} with user: ${PG_USER}"
	@sudo -u postgres createuser -d -r -s $(PG_USER) 2>/dev/null || true
	@sudo -u postgres psql -c "ALTER USER $(PG_USER) PASSWORD '$(PG_PW)'" 2>/dev/null || true
	@sudo -u postgres createdb -O $(PG_USER) $(PG_DB) 2>/dev/null || true
	@sudo -u postgres createdb -O $(PG_USER) $(PG_USER) 2>/dev/null || true

# Documentation targets

.PHONY: docs-install
docs-install: ## Install documentation dependencies
	@echo "Installing documentation dependencies..."
	@cd website && npm install

.PHONY: docs-build
docs-build: docs-integrate-rust docs-install ## Build the documentation website with Rust API docs
	@echo "Building documentation website with Rust API documentation..."
	@cd website && npm run build
	@echo "Documentation built successfully!"
	@echo "Built files are in website/build/"

.PHONY: docs-serve
docs-serve: docs-integrate-rust docs-install ## Serve the documentation website locally with Rust API docs
	@echo "Starting documentation server with Rust API documentation..."
	@echo "Documentation will be available at: http://localhost:$(DOCS_PORT)"
	@cd website && npm start -- --port $(DOCS_PORT)

.PHONY: docs-build-serve
docs-build-serve: docs-build ## Build and serve the documentation website locally with Rust API docs
	@echo "Serving built documentation with Rust API documentation at: http://localhost:$(DOCS_PORT)"
	@cd website && npm run serve -- --port $(DOCS_PORT)

.PHONY: docs-build-only
docs-build-only: docs-install ## Build the documentation website without Rust API docs
	@echo "Building documentation website (without Rust API docs)..."
	@cd website && npm run build
	@echo "Documentation built successfully!"
	@echo "Built files are in website/build/"

.PHONY: docs-serve-only
docs-serve-only: docs-install ## Serve the documentation website locally without Rust API docs
	@echo "Starting documentation server (without Rust API docs)..."
	@echo "Documentation will be available at: http://localhost:$(DOCS_PORT)"
	@cd website && npm start -- --port $(DOCS_PORT)

.PHONY: docs-rust
docs-rust: ## Generate Rust API documentation
	@echo "Generating Rust API documentation..."
	# Using nightly with --enable-index-page to generate workspace index
	# See: https://github.com/rust-lang/cargo/issues/8229
	RUSTDOCFLAGS="--enable-index-page -Zunstable-options -D warnings" \
		cargo +$(NIGHTLY_RUST_VERSION) doc --no-deps \
		--document-private-items --workspace \
		--lib
	@echo "Rust documentation generated in target/doc/"
	@echo "Entry point: target/doc/index.html"

.PHONY: docs-integrate-rust
docs-integrate-rust: docs-rust ## Integrate Rust API documentation into website
	@echo "Integrating Rust API documentation..."
	@mkdir -p website/static/api-docs
	@rm -rf website/static/api-docs/*
	@cp -r target/doc/* website/static/api-docs/
	@echo "Rust API documentation integrated into website/static/api-docs/"


.PHONY: docs-clean
docs-clean: ## Clean documentation build artifacts
	@echo "Cleaning documentation build artifacts..."
	@rm -rf website/build website/.docusaurus website/static/api-docs target/doc
	@echo "Documentation artifacts cleaned!"

# Release management targets

.PHONY: release-validate
release-validate: ## Validate codebase is ready for release
	@website/docs/developers/scripts/release/validate.sh

.PHONY: release-update-version
release-update-version: ## Update version in Cargo.toml files (requires VERSION=x.y.z)
	@if [ -z "$(VERSION)" ]; then \
		echo "Error: VERSION is required. Usage: make release-update-version VERSION=1.2.3"; \
		exit 1; \
	fi
	@website/docs/developers/scripts/release/update-version.sh "$(VERSION)"

.PHONY: release-docker-verify
release-docker-verify: ## Verify multi-arch Docker images are available (requires TAG=version)
	@if [ -z "$(TAG)" ]; then \
		echo "Error: TAG is required. Usage: make release-docker-verify TAG=v1.2.3"; \
		exit 1; \
	fi
	@DOCKER_ORG=$(DOCKER_ORG) website/docs/developers/scripts/release/verify-docker.sh "$(TAG)"

.PHONY: release-create-tag
release-create-tag: ## Create and push git tag (requires TAG=version MESSAGE="description")
	@if [ -z "$(TAG)" ] || [ -z "$(MESSAGE)" ]; then \
		echo "Error: TAG and MESSAGE are required."; \
		echo "Usage: make release-create-tag TAG=v1.2.3 MESSAGE='Release v1.2.3'"; \
		exit 1; \
	fi
	@website/docs/developers/scripts/release/create-tag.sh "$(TAG)" "$(MESSAGE)"

.PHONY: release-merge-back
release-merge-back: ## Merge main back to develop after release
	@website/docs/developers/scripts/release/merge-back.sh

.PHONY: release-help
release-help: ## Show release management commands
	@echo "Release Management Commands:"
	@echo ""
	@echo "  release-validate          - Validate codebase (tests, format, etc.)"
	@echo "  release-update-version    - Update Cargo.toml versions (requires VERSION=x.y.z)"
	@echo "  release-create-tag        - Create and push git tag (requires TAG=vx.y.z MESSAGE='...')"
	@echo "  release-docker-verify     - Verify Docker images (requires TAG=vx.y.z)"
	@echo "  release-merge-back        - Merge main back to develop"
	@echo ""
	@echo "Example workflow:"
	@echo "  make release-validate"
	@echo "  make release-update-version VERSION=1.2.3"
	@echo "  # Create PR, get approval, merge to main"
	@echo "  make release-create-tag TAG=v1.2.3 MESSAGE='Release v1.2.3'"
	@echo "  make release-docker-verify TAG=v1.2.3"
	@echo "  make release-merge-back"
