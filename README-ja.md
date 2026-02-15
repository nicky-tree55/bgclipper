# bgclipper

[![CI](https://github.com/nicky-tree55/bgclipper/actions/workflows/ci.yml/badge.svg)](https://github.com/nicky-tree55/bgclipper/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![Platform](https://img.shields.io/badge/platform-macOS%20%7C%20Windows-blue)]()

🌐 [English](README.md)

**指定した背景色を自動で透明化 — クリップボードから直接。**

## bgclipper とは？

bgclipper はスクリーンショットを透明化してクリップボードに画像を追加するシンプルなアプリケーションです。画像をコピーすると指定した RGB 値のピクセルを即座に透明化し、結果をクリップボードに戻します。透明化した画像をシームレスにそのまま貼り付けられます。

手動編集も追加ステップも不要。コピーして貼るだけ。

## デモ

<!-- TODO: 実装後にデモ GIF を追加 -->
<!-- ![demo](docs/demo.gif) -->

## 機能

- 🎯 **RGB 完全一致** — 指定した 1 色を透明化
- 📋 **クリップボード完結** — ファイル操作不要、クリップボードのみで動作
- 🖥️ **システムトレイ常駐** — バックグラウンドで静かに動作、有効/無効の切替可能
- ⚙️ **シンプルな設定** — TOML 設定ファイル + トレイからの設定 GUI
- 🍎🪟 **クロスプラットフォーム** — macOS (Apple Silicon) / Windows 対応

## インストール

### ビルド済みバイナリ（推奨）

お使いのプラットフォーム向けの最新リリースを [Releases](https://github.com/nicky-tree55/bgclipper/releases) からダウンロードしてください。

| プラットフォーム | ファイル |
|---|---|
| macOS (Apple Silicon) | `bgclipper-aarch64-apple-darwin.tar.gz` |
| Windows | `bgclipper-x86_64-pc-windows-msvc.zip` |

### ソースからビルド

[Rust](https://rustup.rs/) 1.75 以上が必要です。

```bash
git clone https://github.com/nicky-tree55/bgclipper.git
cd bgclipper
cargo build --release
```

## 使い方

1. `bgclipper` を起動 — システムトレイに表示されます。
2. トレイアイコンを右クリックして **設定** を開き、対象の RGB 色を指定します（デフォルト: 白 `255, 255, 255`）。
3. 画像をクリップボードにコピーします（例: スクリーンショット）。
4. 背景色が自動的に透明化されます。
5. お好みのアプリケーションにそのまま貼り付けます。

### 設定

設定は TOML ファイルに保存されます:

- **macOS:** `~/.config/bgclipper/config.toml`
- **Windows:** `%APPDATA%\bgclipper\config.toml`

```toml
[target_color]
r = 255
g = 255
b = 255
```

システムトレイの設定 GUI からも色を変更できます。

## 仕組み

```
┌──────────┐    clipboard    ┌───────────┐    transparent    ┌──────────┐
│  コピー   │ ──── image ───▶ │ bgclipper │ ──── image ────▶ │  貼付け   │
│  (Cmd+C) │                 │  (tray)   │                  │  (Cmd+V) │
└──────────┘                 └───────────┘                  └──────────┘
```

1. OS ネイティブイベントでクリップボードの画像変更を検知
2. 全ピクセルを走査し、対象の RGB 色と照合
3. 一致するピクセルを完全に透明化（アルファ = 0）
4. 処理済み PNG をクリップボードに書き戻し

## コントリビュート

バグ報告、機能提案、プルリクエストなど、あらゆる貢献を歓迎します！

- 開発環境の構築とガイドラインは [CONTRIBUTING.md](CONTRIBUTING.md) をご覧ください。
- 初めての方は [Good First Issues](https://github.com/nicky-tree55/bgclipper/labels/good%20first%20issue) をチェックしてみてください。
- [行動規範](CODE_OF_CONDUCT.md) に従ってください。

## ライセンス

MIT — 詳細は [LICENSE](LICENSE) を参照してください。
