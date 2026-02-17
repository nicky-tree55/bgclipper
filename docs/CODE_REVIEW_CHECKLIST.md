# Code Review Checklist

A self-check checklist for PR authors before requesting review.

## General

- [ ] PR description clearly explains **what** was changed and **why**
- [ ] Changes are scoped to a single concern (no unrelated modifications)
- [ ] No temporary or debug code left behind (e.g., `dbg!`, `println!`, `todo!`)
- [ ] Commit history is clean and each commit has a meaningful message
- [ ] Breaking changes are explicitly noted in the PR description

## Code Quality

- [ ] Code is readable and self-explanatory; comments explain **why**, not **what**
- [ ] Functions and methods are small with a single, clear responsibility
- [ ] Naming follows [RFC 430](https://github.com/rust-lang/rfcs/blob/master/text/0430-finalizing-naming-conventions.md) conventions
- [ ] No deeply nested logic; refactored with early returns, functions, or combinators
- [ ] No code duplication; shared logic is extracted appropriately
- [ ] Magic numbers and strings are replaced with named constants

## Rust-Specific

- [ ] Ownership and borrowing are used correctly; no unnecessary `clone()`
- [ ] `&str` is preferred over `String` for function parameters when ownership is not needed
- [ ] Iterators are used instead of index-based loops where appropriate
- [ ] No unnecessary `unsafe` code; any `unsafe` block is documented with a safety justification
- [ ] Common traits (`Debug`, `Clone`, `PartialEq`, etc.) are implemented where appropriate
- [ ] Public types implement `Debug` at minimum
- [ ] Structs have private fields unless public access is intentionally needed

## Error Handling

- [ ] Errors are handled with `Result<T, E>` and propagated using `?`
- [ ] No `unwrap()` or `expect()` in production code without strong justification
- [ ] No panics in library code; `panic!` is reserved for truly unrecoverable situations
- [ ] Custom error types provide meaningful context and messages
- [ ] `Option<T>` is used appropriately for values that may not exist
- [ ] Error cases are tested

## Performance

- [ ] No unnecessary heap allocations or copies
- [ ] `collect()` is not called prematurely; iterators remain lazy when possible
- [ ] Borrowing and zero-copy operations are preferred over owned data
- [ ] No obvious O(n²) or worse algorithms where a better alternative exists

## Documentation

- [ ] All new public APIs have `///` rustdoc comments
- [ ] Complex logic includes inline comments explaining the approach
- [ ] README is updated if user-facing behavior has changed
- [ ] CHANGELOG is updated for notable changes

## CI / Tooling

- [ ] `cargo fmt` passes with no changes
- [ ] `cargo clippy` passes with no warnings
- [ ] `cargo test` passes (all existing and new tests)
- [ ] `cargo build` completes without warnings
