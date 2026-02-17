# Testing Checklist

A self-check checklist for PR authors to ensure adequate test coverage.

## Test Definitions

This project classifies tests based on the [Small / Medium / Large](https://testing.googleblog.com/2010/12/test-sizes.html) test size model.

### Unit Tests (Small Tests)

Tests that run **within a single process** and have no external dependencies.

- Execute entirely in-process with no I/O, network, or OS resource access.
- Fast and deterministic — suitable for running on every build.
- Located in `#[cfg(test)] mod tests` blocks alongside the source code.
- May use mock implementations of port traits for isolation.
- Run with: `cargo test --lib`

### Integration Tests (Medium Tests)

Tests that may **span multiple processes** or interact with OS-level resources.

- May access the clipboard, file system, or other OS services.
- May launch the application binary as a subprocess and verify its behavior.
- Located in the `tests/` directory at the project root.
- Each `.rs` file in `tests/` is compiled as a separate test binary.
- Run with: `cargo test --test '*'`

> **Note:** Large tests (end-to-end tests involving external services or network) are not currently in scope for this project, as bgclipper operates entirely locally.

## Unit Tests

- [ ] New functions and methods have corresponding unit tests
- [ ] Tests are placed in `#[cfg(test)] mod tests` alongside the code they test
- [ ] Each test has a descriptive name that explains the scenario being tested
- [ ] Tests cover both the happy path and error/failure paths
- [ ] Edge cases are identified and tested (e.g., empty input, boundary values, overflow)

## Integration Tests

- [ ] Integration tests are added in the `tests/` directory when cross-module or cross-process behavior is involved
- [ ] Integration test filenames are descriptive and reflect the feature being tested
- [ ] Tests that require OS resources (clipboard, file system) are properly guarded for CI environments

## Test Quality

- [ ] Tests are independent and do not rely on execution order
- [ ] Tests use meaningful assertions with clear failure messages
- [ ] No `unwrap()` in tests where `?` with a `Result` return type is more appropriate
- [ ] Test data and fixtures are minimal and focused on the scenario
- [ ] Flaky or timing-dependent tests are avoided; deterministic behavior is ensured

## Coverage

- [ ] Modified code paths have test coverage
- [ ] Regression tests are added for bug fixes
- [ ] All existing tests pass (`cargo test`)

## Platform-Specific

- [ ] Platform-specific code (macOS / Windows) is tested or guarded with `#[cfg(target_os)]`
- [ ] Clipboard-related tests handle the absence of a display server gracefully (e.g., CI environments)
