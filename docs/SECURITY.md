# Security Checklist

A self-check checklist for PR authors to ensure security best practices.

## Input Validation

- [ ] All external input (clipboard data, config files, user settings) is validated before use
- [ ] Image data from the clipboard is checked for valid format and reasonable size
- [ ] TOML config parsing handles malformed input gracefully without panicking
- [ ] Numeric inputs (e.g., RGB values) are bounds-checked (0–255)

## Dependencies

- [ ] New dependencies are from well-maintained, reputable crates
- [ ] No unnecessary dependencies are added; prefer the standard library when possible
- [ ] Dependency versions are pinned or use compatible ranges to avoid unexpected breakage
- [ ] `cargo audit` reports no known vulnerabilities

## Unsafe Code

- [ ] No `unsafe` code is introduced without a compelling reason
- [ ] Any `unsafe` block includes a `// SAFETY:` comment explaining the invariants
- [ ] Unsafe code is minimized and wrapped in a safe API

## File System and OS Interaction

- [ ] File paths are constructed safely (no path traversal vulnerabilities)
- [ ] Config file permissions are appropriate (not world-writable)
- [ ] Temporary files are cleaned up and not left behind

## Sensitive Data

- [ ] No secrets, credentials, or API keys are hardcoded or logged
- [ ] Debug output (`dbg!`, `println!`) does not leak sensitive information
- [ ] Error messages do not expose internal implementation details to end users
