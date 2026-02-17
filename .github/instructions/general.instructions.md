# General Instructions
bgclipper is a simple application that makes screenshots transparent and puts the result back on your clipboard. When you copy an image, it instantly replaces pixels matching the specified RGB value with transparency. The transparent image is ready to paste seamlessly wherever you need it.

## Languages and Frameworks
- Rust: The core application is written in Rust, leveraging its performance and safety features.

## Project Structure
- `docs/PROJECT_STRUCTURE.md`: Overview of the repository folder structure and directory purposes.

## Coding Style Guidelines
- `docs/RUST_CODING_STYLE.md`: Guidelines for coding style in Rust.

## Branch Strategy

This project follows **GitHub Flow**:

- `main` is always deployable. Do not commit directly to `main`.
- Create a feature branch for every change, open a Pull Request, and merge after review.

### Branch Naming

Use a prefix that matches the type of change:

| Prefix | Purpose | Example |
|---|---|---|
| `feat/` | New feature | `feat/add-image-processor` |
| `fix/` | Bug fix | `fix/clipboard-read-error` |
| `docs/` | Documentation only | `docs/update-readme` |
| `refactor/` | Code refactoring (no behavior change) | `refactor/extract-config-module` |
| `test/` | Adding or updating tests | `test/add-color-edge-cases` |
| `chore/` | Build, CI, tooling, or maintenance | `chore/update-ci-workflow` |

Rules:
- Use lowercase kebab-case after the prefix (e.g., `feat/add-tray-icon`).
- Keep branch names short and descriptive.

## Code Review Checklist
- `docs/CODE_REVIEW_CHECKLIST.md`: A checklist to follow during code reviews to ensure code quality and consistency.
- `docs/TEST.md`: Testing guidelines and checklist for ensuring adequate test coverage.
- `docs/SECURITY.md`: Security checklist for ensuring security best practices.