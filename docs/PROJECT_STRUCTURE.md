# Project Structure

```
bgclipper/
├── .github/
│   └── instructions/              # Copilot custom instructions
│       ├── general.instructions.md    # General guidelines and doc references
│       └── rust.instructions.md       # Rust-specific coding conventions (applies to *.rs)
├── docs/                          # Project documentation
│   ├── CODE_REVIEW_CHECKLIST.md       # PR self-check checklist
│   ├── PROJECT_STRUCTURE.md           # This file
│   ├── RUST_CODING_STYLE.md           # Rust coding style guide
│   ├── SECURITY.md                    # Security checklist
│   └── TEST.md                        # Testing guidelines and definitions
├── logo/                          # Branding assets
│   ├── logo.drawio                    # Logo source (draw.io)
│   └── logo.svg                       # Logo image
├── src/
│   ├── main.rs                        # Entry point (tray app startup)
│   ├── lib.rs                         # Library crate root (re-exports modules)
│   ├── domain/                        # Domain layer
│   │   ├── mod.rs
│   │   ├── color.rs                   # RGB color value object
│   │   ├── image_processor.rs         # Transparency conversion logic (Domain Service)
│   │   └── port.rs                    # Port traits (ClipboardPort, ConfigPort)
│   ├── application/                   # Application layer
│   │   ├── mod.rs
│   │   └── clipboard_service.rs       # Use case: monitor clipboard → convert → write back
│   ├── infrastructure/                # Infrastructure layer
│   │   ├── mod.rs
│   │   ├── clipboard.rs              # ClipboardPort implementation (OS-native clipboard access)
│   │   └── config.rs                  # ConfigPort implementation (TOML config file read/write)
│   └── presentation/                  # Presentation layer
│       ├── mod.rs
│       └── tray.rs                    # System tray UI and settings dialog
├── tests/                         # Integration tests (cross-process, Medium tests)
│   └── *.rs                           # Each file is a separate test binary
├── config.toml.example            # Example configuration file
├── Cargo.toml                     # Rust package manifest
├── Cargo.lock                     # Dependency lock file
├── LICENSE                        # MIT license
├── README.md                      # Project documentation (English)
└── README-ja.md                   # Project documentation (Japanese)
```

## Directory Descriptions

| Directory | Purpose |
|---|---|
| `.github/instructions/` | Copilot custom instructions that are automatically applied during code generation and review. |
| `docs/` | Project documentation including coding style, review checklists, and guidelines. |
| `logo/` | Logo and branding assets. |
| `src/` | Rust source code. `main.rs` is the binary entry point; `lib.rs` is the library crate root. |
| `src/domain/` | Core business logic with no external dependencies. Contains value objects, domain services, and port traits for dependency inversion. |
| `src/application/` | Use cases that orchestrate domain logic. Depends on domain layer only (via port traits). |
| `src/infrastructure/` | Implementations of port traits. Handles OS clipboard access, file I/O, and configuration parsing. |
| `src/presentation/` | User-facing components. System tray icon, settings dialog, and event handling. |
| `tests/` | Integration tests (Medium tests) that may span multiple processes or interact with OS resources. |

## Architecture

The project follows a **layered architecture** with dependency inversion:

```
presentation → application → domain ← infrastructure
```

- **domain** defines port traits (`ClipboardPort`, `ConfigPort`) as interfaces.
- **infrastructure** implements these traits with concrete OS-level adapters.
- **application** depends only on domain traits, enabling easy testing with mock implementations.
- **presentation** wires everything together and handles UI events.

### Binary vs Library Separation

- `main.rs` — Binary crate. Minimal entry point that initializes dependencies and starts the tray app.
- `lib.rs` — Library crate. Re-exports all modules. Used by `main.rs` and integration tests.
