# Changelog

## [2.1.0] — 2026-05-07

### Added
- **Tauri Desktop App** — Native cross-platform shell with system tray, global hotkey (Ctrl+Shift+Space), and auto-updater
- **Rust BibleEngine** — Full port of Python `bible_engine.py` to Rust with `rusqlite` + FTS5
- **CLI Companion** — Standalone Rust binary (`coders-bible-cli`) with colored output, JSON mode, and personal snippet layer
- **PWA Support** — `manifest.webmanifest`, service worker with cache-first offline strategy
- **VS Code Extension v2.1** — Analyze command, right-click context menu, Rust CLI bridge with Python fallback
- **CI/CD** — GitHub Actions workflows for Tauri (Win/Mac/Linux) and CLI release builds with SHA-256 checksums
- **Unit Tests** — 10 deterministic tests covering language detection, safety analysis, search, and stats
- **Icons** — Ω logo generated for Tauri (32x32, 128x128, 256x256) and PWA (72–512)

### Changed
- Frontend migrated from Flask `fetch()` to Tauri `invoke()` with graceful browser fallback
- Search ranking uses native SQLite FTS5 `rank` instead of `bm25()` for consistency
- VS Code extension now prefers Rust CLI, falls back to Python `cb.py`

### Security
- Zero telemetry — no network calls except explicit updater checks
- Zero AI — all analysis is deterministic regex-based
- CSP hardened in Tauri config

## [2.0.0] — Previous
- Original Python Flask backend + vanilla JS frontend
- 67,213 fragments across 28 domains
- FTS5 full-text search with domain-aware routing
- Language detection via regex fingerprints
- Safety classification with grounded descriptions
