# Frontend Makefile

.PHONY: help
help: ## Display this help message
	@echo "Frontend Makefile - Available targets:"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | \
		awk 'BEGIN {FS = ":.*?## "}; \
		{printf "\033[36m%-20s\033[0m %s\n", $$1, $$2}'

.PHONY: build
build: ## Build the frontend
	npx ng build

.PHONY: build-fuzzing
build-fuzzing: ## Build the frontend with fuzzing configuration
	npx ng build --configuration fuzzing
	cp dist/frontend/browser/assets/environments/fuzzing.js \
		dist/frontend/browser/assets/environments/env.js

.PHONY: build-local
build-local: ## Build the frontend with local configuration
	npx ng build --configuration local
	cp dist/frontend/browser/assets/environments/local.js \
		dist/frontend/browser/assets/environments/env.js

.PHONY: build-production-sentry
build-production-sentry: ## Build the frontend for production with Sentry sourcemaps
	npx ng build --configuration production
	cp dist/frontend/browser/assets/environments/production.js \
		dist/frontend/browser/assets/environments/env.js
	$(MAKE) sentry-sourcemaps

.PHONY: build-production
build-production: ## Build the frontend with production configuration
	npx ng build --configuration production
	cp dist/frontend/browser/assets/environments/production.js \
		dist/frontend/browser/assets/environments/env.js

.PHONY: build-webnode
build-webnode: ## Build the frontend with webnode configuration
	npx ng build --configuration webnode-local
	cp dist/frontend/browser/assets/environments/webnode.js \
		dist/frontend/browser/assets/environments/env.js

.PHONY: build-staging
build-staging: ## Build the frontend with staging configuration
	npx ng build --configuration production
	cp dist/frontend/browser/assets/environments/staging.js \
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

.PHONY: prebuild
prebuild: ## Run prebuild script to update version
	node scripts/update-frontend-version.js

.PHONY: prettify
prettify: ## Format HTML, SCSS, TypeScript, and JavaScript files with Prettier
	npx prettier --write 'src/**/*.{ts,js,html,scss,css,json}'

.PHONY: rebuild
rebuild: clean install-deps build ## Clean, reinstall dependencies, and rebuild

.PHONY: sentry-sourcemaps
sentry-sourcemaps: ## Upload sourcemaps to Sentry
	sentry-cli sourcemaps inject --org openmina-uv --project openmina \
		./dist/frontend/browser
	sentry-cli sourcemaps upload --org openmina-uv --project openmina \
		./dist/frontend/browser

.PHONY: start
start: install-deps ## Start the development server
	npx ng serve --configuration local --open

.PHONY: start-bundle
start-bundle: ## Serve the built bundle
	serve dist/frontend/browser -s -l 4200

.PHONY: start-dev-mobile
start-dev-mobile: ## Start the development server for mobile (accessible on network)
	npx ng serve --configuration local --host 0.0.0.0

.PHONY: start-fuzzing
start-fuzzing: install-deps ## Start the fuzzing build and serve
	npx ng build --configuration fuzzing
	$(MAKE) start-bundle

.PHONY: start-local
start-local: ## Start the development server with local configuration
	npx ng serve --configuration local

.PHONY: start-production
start-production: ## Start the development server with production configuration
	npx ng serve --configuration production

.PHONY: start-webnode
start-webnode: install-deps ## Start the webnode development server
	npx ng serve --configuration webnode-local --open

.PHONY: test
test: ## Run Cypress tests
	npx cypress open --config baseUrl=http://localhost:4200

.PHONY: test-headless
test-headless: ## Run Cypress tests in headless mode
	npx cypress run --headless --config baseUrl=http://localhost:4200
