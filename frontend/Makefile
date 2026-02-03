# Frontend Makefile

# These are threaded from the outside env into build-webnode[-production]
MINA_WEBNODE_SEED_URLS ?=
MINA_WEBNODE_BOOTNODES ?=

.PHONY: help
help: ## Display this help message
	@echo "Frontend Makefile - Available targets:"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | \
		awk 'BEGIN {FS = ":.*?## "}; \
		{printf "\033[36m%-20s\033[0m %s\n", $$1, $$2}'

.PHONY: build
build: ## Build the frontend
	npx ng build

.PHONY: build-local
build-local: ## Build the frontend with local configuration
	npx ng build --configuration local
	cp dist/frontend/browser/assets/environments/local.js \
		dist/frontend/browser/assets/environments/env.js

.PHONY: build-production
build-production: ## Build the frontend with production configuration
	npx ng build --configuration production
	cp dist/frontend/browser/assets/environments/production.js \
		dist/frontend/browser/assets/environments/env.js

.PHONY: build-webnode
build-webnode: ## Build the frontend with webnode configuration
	npx ng build \
	  --define MINA_WEBNODE_SEED_URLS="\"$(MINA_WEBNODE_SEED_URLS)\"" \
	  --define MINA_WEBNODE_BOOTNODES="\"$(MINA_WEBNODE_BOOTNODES)\"" \
	  --configuration webnode-local
	cp dist/frontend/browser/assets/environments/webnode.js \
		dist/frontend/browser/assets/environments/env.js

.PHONY: build-webnode-production
build-webnode-production: ## Build the frontend with webnode configuration and optimizations
	npx ng build \
	  --define MINA_WEBNODE_SEED_URLS="\"$(MINA_WEBNODE_SEED_URLS)\"" \
	  --define MINA_WEBNODE_BOOTNODES="\"$(MINA_WEBNODE_BOOTNODES)\"" \
	  --configuration webnode-production
	cp dist/frontend/browser/assets/environments/webnode.js \
		dist/frontend/browser/assets/environments/env.js

.PHONY: check-prettify
check-prettify: ## Check if files are formatted with Prettier
	npx prettier --check 'src/**/*.{ts,js,html,scss,css,json}'

.PHONY: clean
clean: ## Clean build artifacts and node_modules
	rm -rf dist node_modules

.PHONY: copy-env
copy-env: ## Copy webnode.js to env.js
	cp dist/frontend/browser/assets/environments/webnode.js \
		dist/frontend/browser/assets/environments/env.js

.PHONY: install-deps
install-deps: ## Install npm dependencies (alias)
	npm install

.PHONY: prettify
prettify: ## Format HTML, SCSS, TypeScript, and JavaScript files with Prettier
	npx prettier --write 'src/**/*.{ts,js,html,scss,css,json}'

.PHONY: rebuild
rebuild: clean install-deps build ## Clean, reinstall dependencies, and rebuild

.PHONY: start
start: install-deps ## Start the development server
	npx ng serve --configuration local --open

.PHONY: start-bundle
start-bundle: ## Serve the built bundle
	serve dist/frontend/browser -s -l 4200

.PHONY: start-dev-mobile
start-dev-mobile: ## Start the development server for mobile (accessible on network)
	npx ng serve --configuration development --host 0.0.0.0

.PHONY: start-local
start-local: ## Start the development server with local configuration
	npx ng serve --configuration local

.PHONY: start-production
start-production: ## Start the development server with production configuration
	npx ng serve --configuration production

.PHONY: start-webnode
start-webnode: install-deps ## Start the webnode development server
	npx ng serve --configuration webnode-local --open

.PHONY: start-webnode-production
start-webnode-production: install-deps ## Start the webnode development server with optimizations
	npx ng serve --configuration webnode-production --open

.PHONY: test
test: ## Run Cypress tests
	npx cypress open --config baseUrl=http://localhost:4200

.PHONY: test-headless
test-headless: ## Run Cypress tests in headless mode
	npx cypress run --headless --config baseUrl=http://localhost:4200
