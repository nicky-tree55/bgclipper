# bgclipper

[![CI](https://github.com/nicky-tree55/bgclipper/actions/workflows/ci.yml/badge.svg)](https://github.com/nicky-tree55/bgclipper/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![Platform](https://img.shields.io/badge/platform-macOS%20%7C%20Windows-blue)]()

🌐 [日本語](README-ja.md)

**Automatically make a specific background color transparent — right from your clipboard.**

## What is bgclipper?

bgclipper is a simple application that makes screenshots transparent and puts the result back on your clipboard. When you copy an image, it instantly replaces pixels matching the specified RGB value with transparency. The transparent image is ready to paste seamlessly wherever you need it.

No manual editing. No extra steps. Just copy and paste.

## Demo

<!-- TODO: Add demo GIF after implementation -->
<!-- ![demo](docs/demo.gif) -->

## Features

- 🎯 **Exact RGB match** — Specify one color to make transparent
- 📋 **Clipboard-driven** — No file I/O; works entirely through clipboard
- 🖥️ **System tray** — Runs quietly in the background with enable/disable toggle
- ⚙️ **Simple config** — TOML file + tray settings GUI
- 🍎🪟 **Cross-platform** — macOS (Apple Silicon) and Windows

## Installation

### Pre-built binaries (Recommended)

Download the latest release for your platform from [Releases](https://github.com/nicky-tree55/bgclipper/releases).

| Platform | File |
|---|---|
| macOS (Apple Silicon) | `bgclipper-aarch64-apple-darwin.tar.gz` |
| Windows | `bgclipper-x86_64-pc-windows-msvc.zip` |

### Build from source

Requires [Rust](https://rustup.rs/) 1.75+.

```bash
git clone https://github.com/nicky-tree55/bgclipper.git
cd bgclipper
cargo build --release
```

## Usage

1. Launch `bgclipper` — it appears in your system tray.
2. Right-click the tray icon to open **Settings** and set your target RGB color (default: white `255, 255, 255`).
3. Copy any image to your clipboard (e.g. a screenshot).
4. The background color is automatically made transparent.
5. Paste the image into your favorite application.

### Configuration

Settings are stored in a TOML file:

- **macOS:** `~/.config/bgclipper/config.toml`
- **Windows:** `%APPDATA%\bgclipper\config.toml`

```toml
[target_color]
r = 255
g = 255
b = 255
```

You can also edit the color from the system tray settings GUI.

## How It Works

```
┌──────────┐    clipboard    ┌───────────┐    transparent    ┌──────────┐
│  Copy    │ ──── image ───▶ │ bgclipper │ ──── image ────▶ │  Paste   │
│  (Cmd+C) │                 │  (tray)   │                  │  (Cmd+V) │
└──────────┘                 └───────────┘                  └──────────┘
```

1. Detects clipboard image change via OS-native events
2. Scans every pixel for the target RGB color
3. Sets matching pixels to fully transparent (alpha = 0)
4. Writes the processed PNG back to the clipboard

## Contributing

Contributions are welcome! Whether it's a bug report, feature request, or pull request — we appreciate your help.

- See [CONTRIBUTING.md](CONTRIBUTING.md) for development setup and guidelines.
- Check out [Good First Issues](https://github.com/nicky-tree55/bgclipper/labels/good%20first%20issue) if you're looking for a place to start.
- Please follow our [Code of Conduct](CODE_OF_CONDUCT.md).

## License

MIT — see [LICENSE](LICENSE) for details.