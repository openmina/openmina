---
sidebar_position: 6
---

# Code Coverage

This guide explains how to generate, analyze, and use code coverage reports in
the OpenMina project.

## Overview

OpenMina uses Rust's built-in LLVM-based code coverage instrumentation to
generate coverage reports. This provides accurate line-level and branch-level
coverage information for all Rust code in the workspace.

## Quick Start

### Setup Coverage Tools

First, install the required coverage tools:

```bash
make setup-coverage-tools
```

This installs:

- `llvm-tools-preview` Rust component for LLVM coverage tools
- `grcov` for processing coverage data and generating reports

### Generate Coverage Reports

#### Basic Coverage (Fast)

For quick coverage checks during development:

```bash
make test-coverage
make coverage-report
```

This runs tests for libraries and basic test files, then generates an HTML
report you can view in your browser.

#### Comprehensive Coverage (Complete)

For complete coverage analysis:

```bash
make test-with-coverage
make coverage-report
```

This runs all tests including integration tests and binaries, providing more
complete coverage data.

### View Reports

After generating coverage, open the HTML report:

```bash
# On Linux
xdg-open target/coverage/html/index.html

# On macOS
open target/coverage/html/index.html
```

## Coverage Commands Reference

| Command                     | Description                                      |
| --------------------------- | ------------------------------------------------ |
| `make setup-coverage-tools` | Install required coverage tools                  |
| `make test-coverage`        | Run basic tests with coverage (fast)             |
| `make test-with-coverage`   | Run comprehensive tests with coverage (complete) |
| `make coverage-report`      | Generate HTML coverage report                    |
| `make coverage-lcov`        | Generate LCOV report for CI/codecov              |
| `make coverage-summary`     | Display coverage summary in terminal             |
| `make coverage-clean`       | Clean all coverage data and reports              |

## Understanding Coverage Reports

### HTML Reports

The HTML report (`target/coverage/html/index.html`) provides:

- **Overview**: Overall coverage percentage for the project
- **File List**: Coverage breakdown by source file
- **Line Coverage**: Which lines are covered (green) or uncovered (red)
- **Branch Coverage**: Which code branches are taken during tests
- **Function Coverage**: Which functions are called during tests

### Coverage Metrics

- **Line Coverage**: Percentage of executable lines that are executed by tests
- **Branch Coverage**: Percentage of conditional branches that are taken during
  tests
- **Function Coverage**: Percentage of functions that are called during tests

### Coverage Thresholds

The project aims for:

- **Minimum**: 70% line coverage
- **Target**: 85%+ line coverage
- **Excellent**: 95%+ line coverage

## CI Integration

### Nightly Coverage

Coverage reports are automatically generated nightly and uploaded to
[Codecov](https://app.codecov.io/gh/o1-labs/openmina):

- **Schedule**: Daily at 2:00 AM UTC
- **Types**: Both basic and comprehensive coverage
- **Reports**: Uploaded to Codecov and stored as artifacts

### Manual Trigger

You can manually trigger coverage analysis through GitHub Actions:

1. Go to the "Actions" tab in the GitHub repository
2. Select "Code Coverage (Nightly)" workflow
3. Click "Run workflow"
4. Choose coverage type (basic or comprehensive)

## Best Practices

### Writing Testable Code

1. **Write Unit Tests**: Test individual functions and modules
2. **Test Edge Cases**: Include tests for error conditions and boundary cases
3. **Mock Dependencies**: Use mocks for external dependencies in tests
4. **Keep Functions Small**: Smaller functions are easier to test completely

### Improving Coverage

1. **Identify Gaps**: Use coverage reports to find untested code
2. **Prioritize Critical Code**: Focus on testing important business logic first
3. **Test Error Paths**: Ensure error handling code is covered
4. **Add Integration Tests**: Test how components work together

### Excluding Code from Coverage

Some code should be excluded from coverage analysis:

- **Test Code**: Files ending in `*test*.rs`, `tests.rs`
- **Generated Code**: Auto-generated code that doesn't need testing
- **Platform-Specific Code**: Code that only runs on specific platforms
- **Debug/Development Code**: Code only used for debugging

Exclusions are configured in `codecov.yml`.

## Troubleshooting

### Coverage Tools Not Found

If you get errors about missing tools:

```bash
# Reinstall coverage tools
make setup-coverage-tools

# Verify installation
rustup component list --installed | grep llvm-tools
cargo install --list | grep grcov
```

### Empty Coverage Reports

If reports show no coverage:

1. Ensure tests are actually running: `cargo test --workspace --lib`
2. Check that `RUSTFLAGS` environment variable isn't overriding coverage flags
3. Verify coverage data files exist: `ls target/coverage/*.profraw`

### Build Errors with Coverage

Coverage instrumentation can sometimes cause build issues:

```bash
# Clean and rebuild
make coverage-clean
cargo clean
make test-coverage
```

### Slow Coverage Generation

Coverage builds are slower than regular builds due to instrumentation:

- Use `test-coverage` (basic) for faster iteration during development
- Use `test-with-coverage` (comprehensive) for complete analysis
- Consider running coverage on specific packages:
  `cargo test -p <package> --lib`

## Example Workflow

Here's a typical development workflow using coverage:

```bash
# 1. Install tools (one time)
make setup-coverage-tools

# 2. During development - quick coverage check
make test-coverage
make coverage-summary

# 3. Before committing - comprehensive coverage
make test-with-coverage
make coverage-report

# 4. View detailed report
open target/coverage/html/index.html

# 5. Clean up when done
make coverage-clean
```

## Integration with IDEs

### VS Code

Install the "Coverage Gutters" extension to view coverage directly in the
editor:

1. Install the extension
2. Generate LCOV report: `make coverage-lcov`
3. Open the command palette (Ctrl+Shift+P)
4. Run "Coverage Gutters: Display Coverage"
5. Point to `target/coverage/lcov.info`

### Other IDEs

Most IDEs support LCOV format coverage files. Generate the LCOV report with
`make coverage-lcov` and configure your IDE to read `target/coverage/lcov.info`.
