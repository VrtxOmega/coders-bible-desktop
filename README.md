# Coder's Bible Desktop v2.1

> **Sovereign knowledge engine. Zero AI. Zero network. Pure signal.**

The Coder's Bible is a fully offline developer knowledge base containing **67,213 curated code fragments** across **25 domains** — from Python and Rust to Docker, Kubernetes, Nginx, and systemd. This repository contains the **Tauri desktop app** and **PWA assets**. The companion CLI lives at [`coders-bible-cli`](https://github.com/VrtxOmega/coders-bible-cli); the harvest pipeline + web frontend at [`the-coders-bible`](https://github.com/VrtxOmega/the-coders-bible).

---

## Download

Pre-built installers for every release are on the [releases page](https://github.com/VrtxOmega/coders-bible-desktop/releases):

| Platform | Asset |
|---|---|
| Windows | `Coder.s.Bible_2.1.0_x64_en-US.msi` |
| macOS (Apple Silicon) | `Coder.s.Bible_2.1.0_aarch64.dmg` |
| macOS (Intel) | `Coder.s.Bible_2.1.0_x64.dmg` |
| Debian / Ubuntu | `Coder.s.Bible_2.1.0_amd64.deb` |
| Fedora / RHEL | `Coder.s.Bible-2.1.0-1.x86_64.rpm` |
| Other Linux | `Coder.s.Bible_2.1.0_amd64.AppImage` |

> **Windows:** the binary is currently unsigned (no code-signing cert yet). Click "More info → Run anyway" if SmartScreen flags it.
>
> **macOS:** the .dmg is ad-hoc signed. Right-click the app the first time you launch it → "Open" to bypass Gatekeeper.

---

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│  Coder's Bible Ecosystem v2.1                                │
├─────────────────────────────────────────────────────────────┤
│  Tauri Desktop  │  PWA  │  VS Code Extension  │  CLI        │
│     (Rust)      │ (Web) │      (Node.js)      │  (Rust)     │
├─────────────────────────────────────────────────────────────┤
│           Shared: bible_engine.rs (Rust)                    │
│           Shared: coders_bible.db (SQLite + FTS5)           │
└─────────────────────────────────────────────────────────────┘
```

---

## Quick Start

### CLI (Ready Now)

```bash
cd coders-bible-cli
cargo build --release
./target/release/coders-bible-cli "docker compose" --limit 5
./target/release/coders-bible-cli analyze "chmod +x deploy.sh"
```

### Desktop App

```bash
cd src-tauri
cargo tauri dev      # dev mode
cargo tauri build    # release binaries
```

### PWA

Serve `src/` with any static file server:
```bash
cd src
python3 -m http.server 8080
```

Then open `http://localhost:8080` and install via browser prompt.

---

## Features

| Feature | Desktop | PWA | VS Code | CLI |
|---------|---------|-----|---------|-----|
| FTS5 Search | ✅ | ✅* | ✅ | ✅ |
| Analyze Snippet | ✅ | ✅* | ✅ | ✅ |
| Language Detection | ✅ | ✅* | ✅ | ✅ |
| Safety Assessment | ✅ | ✅* | ✅ | ✅ |
| Semantic Decomposition | ✅ | ✅* | ✅ | ✅ |
| Domain Filtering | ✅ | ✅* | ✅ | ✅ |
| Offline-First | ✅ | ✅ | ✅ | ✅ |
| Global Hotkey (Ctrl+Shift+Space) | ✅ | — | ✅ | — |
| System Tray | ✅ | — | — | — |
| Auto-Update | ✅ | — | — | — |

\* PWA requires a running backend or uses the API if hosted.

---

## Tech Stack

- **Backend Engine**: Rust + `rusqlite` (bundled-full, FTS5 enabled)
- **Desktop Shell**: Tauri v2 (WebKit/WebView2)
- **Frontend**: Vanilla HTML5/CSS3/JS (no build step)
- **Database**: SQLite 3 with FTS5 virtual table
- **CLI**: Single Rust binary, ~3 MB

---

## Deterministic Build

The CLI builds deterministically via GitHub Actions:

```yaml
# .github/workflows/cli.yml
# Matrix: ubuntu-latest, macos-latest, windows-latest
# Artifacts: cb-linux-x64, cb-macos-x64, cb-windows-x64.exe
# Checksums: SHA-256
```

The Tauri desktop app builds via:

```yaml
# .github/workflows/release.yml
# Matrix: ubuntu-22.04, macos-latest (aarch64 + x86_64), windows-latest
# Targets: .deb, .rpm, .AppImage, .dmg, .msi
```

---

## Privacy

- **Zero telemetry** — no analytics, no crash reporters, no phoning home
- **Zero network** — all search and analysis happens locally on your machine
- **Zero AI** — deterministic regex-based language detection, no LLM calls
- Database stays in your app data directory; never uploaded

---

## License

MIT — VrtxOmega
