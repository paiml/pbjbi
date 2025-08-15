.PHONY: all build test clean format lint check help install dev-install
.PHONY: test-unit test-integration test-all test-fast coverage
.PHONY: quality-gate pre-commit audit fix doc bench
.PHONY: ci-test ci-build release

# Default target
all: format lint build test

# Help target
help:
	@echo "PBJBI - Pragmatic Business Intelligence Makefile"
	@echo "================================================"
	@echo ""
	@echo "Common targets:"
	@echo "  make build          - Build all crates in debug mode"
	@echo "  make test           - Run all tests"
	@echo "  make format         - Format code with rustfmt"
	@echo "  make lint           - Run clippy linter"
	@echo "  make clean          - Clean build artifacts"
	@echo ""
	@echo "Quality targets:"
	@echo "  make quality-gate   - Run all quality checks (format, lint, test)"
	@echo "  make pre-commit     - Run pre-commit checks"
	@echo "  make audit          - Run security audit"
	@echo "  make coverage       - Generate test coverage report"
	@echo ""
	@echo "Test targets:"
	@echo "  make test-unit      - Run unit tests only"
	@echo "  make test-fast      - Run fast tests (<10s)"
	@echo "  make test-all       - Run all tests including slow ones"
	@echo ""
	@echo "Development:"
	@echo "  make dev-install    - Install development dependencies"
	@echo "  make doc            - Generate documentation"
	@echo "  make bench          - Run benchmarks"
	@echo "  make fix            - Auto-fix code issues"

# Build targets
build:
	@echo "🔨 Building all crates..."
	@cargo build --all-features

build-release:
	@echo "🚀 Building release binaries..."
	@cargo build --release --all-features

# Format targets
format:
	@echo "🎨 Formatting code..."
	@cargo fmt --all
	@echo "✅ Code formatted"

format-check:
	@echo "🔍 Checking code formatting..."
	@cargo fmt --all -- --check

# Lint targets
lint:
	@echo "🔍 Running clippy..."
	@cargo clippy --all-targets --all-features -- -D warnings
	@echo "✅ Linting passed"

lint-fix:
	@echo "🔧 Fixing clippy warnings..."
	@cargo clippy --all-targets --all-features --fix --allow-dirty --allow-staged

# Test targets
test: test-unit

test-unit:
	@echo "🧪 Running unit tests..."
	@cargo test --lib --all-features
	@echo "✅ Unit tests passed"

test-integration:
	@echo "🧪 Running integration tests..."
	@cargo test --test '*' --all-features

test-all:
	@echo "🧪 Running all tests..."
	@cargo test --all-features --workspace
	@echo "✅ All tests passed"

test-fast:
	@echo "⚡ Running fast tests..."
	@cargo test --lib --all-features -- --skip slow
	@echo "✅ Fast tests passed"

test-doc:
	@echo "📚 Testing documentation examples..."
	@cargo test --doc --all-features

# Individual crate tests
test-core:
	@echo "🧪 Testing core crate..."
	@cargo test --package pbjbi-core --all-features

test-symbolic:
	@echo "🧪 Testing symbolic crate..."
	@cargo test --package pbjbi-symbolic --all-features

test-automl:
	@echo "🧪 Testing automl crate..."
	@cargo test --package pbjbi-automl --all-features

# Coverage target
coverage:
	@echo "📊 Generating test coverage..."
	@cargo install cargo-llvm-cov --quiet 2>/dev/null || true
	@cargo llvm-cov --all-features --workspace --html
	@cargo llvm-cov --all-features --workspace --lcov --output-path lcov.info
	@cargo llvm-cov report
	@echo "📊 Coverage report generated in target/llvm-cov/html/index.html"

# Quality gate - must pass before commit
quality-gate: format-check lint test-fast
	@echo "✅ Quality gate passed!"

# Pre-commit hook
pre-commit: format lint-fix test-fast
	@echo "✅ Pre-commit checks passed!"

# Check everything
check: format-check lint test-all audit
	@echo "✅ All checks passed!"

# Security audit
audit:
	@echo "🔒 Running security audit..."
	@cargo audit || (echo "⚠️  Security vulnerabilities found. Run 'cargo audit fix' to attempt fixes." && exit 1)
	@echo "✅ Security audit passed"

# Auto-fix issues
fix:
	@echo "🔧 Auto-fixing issues..."
	@cargo fmt --all
	@cargo clippy --all-targets --all-features --fix --allow-dirty --allow-staged
	@cargo fix --allow-dirty --allow-staged
	@echo "✅ Auto-fix complete"

# Clean build artifacts
clean:
	@echo "🧹 Cleaning build artifacts..."
	@cargo clean
	@rm -rf target/
	@rm -f Cargo.lock
	@echo "✅ Clean complete"

# Documentation
doc:
	@echo "📚 Generating documentation..."
	@cargo doc --all-features --no-deps --open

doc-all:
	@echo "📚 Generating documentation with dependencies..."
	@cargo doc --all-features --open

# Benchmarks
bench:
	@echo "📊 Running benchmarks..."
	@cargo bench --all-features

bench-core:
	@echo "📊 Running core benchmarks..."
	@cargo bench --package pbjbi-core

# Installation
install:
	@echo "📦 Installing pbjbi..."
	@cargo install --path server --force
	@echo "✅ pbjbi installed to ~/.cargo/bin/"

dev-install:
	@echo "📦 Installing development dependencies..."
	@cargo install cargo-audit --quiet 2>/dev/null || true
	@cargo install cargo-llvm-cov --quiet 2>/dev/null || true
	@cargo install cargo-criterion --quiet 2>/dev/null || true
	@cargo install cargo-nextest --quiet 2>/dev/null || true
	@echo "✅ Development dependencies installed"

# CI/CD targets
ci-test: format-check lint test-fast
	@echo "✅ CI tests passed"

ci-build: ci-test build
	@echo "✅ CI build complete"

# Release build
release: quality-gate build-release
	@echo "🎉 Release build complete"

# Watch for changes and run tests
watch:
	@echo "👁️ Watching for changes..."
	@cargo watch -x 'test --lib'

# Run the CLI
run:
	@cargo run --bin pbjbi -- $(ARGS)

# Serve MCP
serve:
	@cargo run --bin pbjbi -- serve

# Statistical analysis of code complexity
analyze-complexity:
	@echo "📊 Analyzing code complexity..."
	@find . -name "*.rs" -not -path "./target/*" | xargs wc -l | sort -rn | head -20

# Count lines of code
loc:
	@echo "📊 Lines of code:"
	@tokei . --exclude target --exclude Cargo.lock

# Toyota Way quality checks
toyota-check:
	@echo "🏭 Running Toyota Way quality checks..."
	@echo "Checking for technical debt markers..."
	@! grep -r "TODO\|FIXME\|HACK\|XXX" --include="*.rs" . 2>/dev/null || (echo "❌ Found technical debt markers" && exit 1)
	@echo "✅ No technical debt markers found"
	@echo "Checking for unwrap() usage..."
	@! grep -r "\.unwrap()" --include="*.rs" core/ symbolic/ automl/ connectors/ 2>/dev/null || (echo "⚠️  Found unwrap() usage - consider using proper error handling" && exit 0)
	@echo "✅ Toyota Way checks complete"

# PMAT-style quality report
quality-report:
	@echo "📊 PBJBI Quality Report"
	@echo "======================"
	@echo ""
	@echo "Test Coverage:"
	@cargo llvm-cov report --quiet 2>/dev/null || echo "  Run 'make coverage' to generate"
	@echo ""
	@echo "Code Statistics:"
	@echo -n "  Total Rust files: "
	@find . -name "*.rs" -not -path "./target/*" | wc -l
	@echo -n "  Total lines: "
	@find . -name "*.rs" -not -path "./target/*" | xargs wc -l | tail -1 | awk '{print $$1}'
	@echo ""
	@echo "Crate Summary:"
	@echo "  • pbjbi-core: Statistical foundations"
	@echo "  • pbjbi-symbolic: Symbolic AI engine"
	@echo "  • pbjbi-automl: AutoML pipelines"
	@echo "  • pbjbi-connectors: Data connectors"
	@echo "  • pbjbi-server: MCP server & CLI"

# Quick development cycle
dev: format lint-fix test-fast
	@echo "✅ Development cycle complete"

# Default for make with no target
.DEFAULT_GOAL := help