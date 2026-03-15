<p align="center">
  <img src="logo/logo.svg" alt="bgclipper logo" width="200">
</p>

# bgclipper

[![CI](https://github.com/nicky-tree55/bgclipper/actions/workflows/ci.yml/badge.svg)](https://github.com/nicky-tree55/bgclipper/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![Platform](https://img.shields.io/badge/platform-macOS%20%7C%20Windows-blue)]()

🌐 [日本語](README-ja.md)

**Make a specific background color transparent with a hotkey — right from your clipboard.**

## What is bgclipper?

bgclipper is a simple application that makes screenshots transparent and puts the result back on your clipboard. When you copy an image, it instantly replaces pixels matching the specified RGB value with transparency. The transparent image is ready to paste seamlessly wherever you need it.

No manual editing. No extra steps. Just copy and paste.

## Demo

<!-- TODO: Add demo GIF after implementation -->
<!-- ![demo](docs/demo.gif) -->

## Features

- 🎯 **Exact RGB match** — Specify one color to make transparent
- 📋 **Clipboard-driven** — No file I/O; works entirely through clipboard
- ⌨️ **Hotkey triggered** — Press a shortcut key to process the clipboard image on demand
- 🖥️ **System tray** — Runs quietly in the background
- ⚙️ **Simple config** — TOML file for target color and hotkey
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
2. Copy any image to your clipboard (e.g. a screenshot).
3. Press **Cmd+Shift+T** (macOS) or **Ctrl+Shift+T** (Windows) to make the background color transparent.
4. Paste the image into your favorite application.

> **Tip:** The hotkey works on any image already on the clipboard — including screenshots taken before launching bgclipper.

### Configuration

Settings are stored in a TOML file:

- **macOS:** `~/.config/bgclipper/config.toml`
- **Windows:** `%APPDATA%\bgclipper\config.toml`

```toml
[target_color]
r = 255
g = 255
b = 255
hotkey = "CmdOrCtrl+Shift+T"
```

- `target_color` — The RGB color to make transparent (default: white).
- `hotkey` — The global shortcut key to trigger processing. If omitted or invalid, defaults to `CmdOrCtrl+Shift+T`.

## How It Works

```
┌──────────┐    clipboard    ┌───────────┐    transparent    ┌──────────┐
│  Copy    │ ──── image ───▶ │ bgclipper │ ──── image ────▶ │  Paste   │
│  (Cmd+C) │                 │ (⌨️ hotkey)│                  │  (Cmd+V) │
└──────────┘                 └───────────┘                  └──────────┘
```

1. User presses the configured hotkey (default: Cmd+Shift+T)
2. Reads the clipboard image
3. Scans every pixel for the target RGB color
4. Sets matching pixels to fully transparent (alpha = 0)
5. Writes the processed PNG back to the clipboard

## Contributing

Contributions are welcome! Whether it's a bug report, feature request, or pull request — we appreciate your help.

- See [CONTRIBUTING.md](CONTRIBUTING.md) for development setup and guidelines.
- Check out [Good First Issues](https://github.com/nicky-tree55/bgclipper/labels/good%20first%20issue) if you're looking for a place to start.
- Please follow our [Code of Conduct](CODE_OF_CONDUCT.md).

## License

MIT — see [LICENSE](LICENSE) for details.