#!/usr/bin/env bash
set -euo pipefail

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║  Coder's Bible v2.1 — Ecosystem Verification Script         ║"
echo "╚══════════════════════════════════════════════════════════════╝"

FAIL=0

# ── CLI ───────────────────────────────────────────────────────
echo ""
echo "▶ CLI Build & Test"
cd "$(dirname "$0")/../coders-bible-cli"
if cargo test --quiet 2>/dev/null; then
    echo "  ✅ CLI tests pass"
else
    echo "  ❌ CLI tests failed"
    FAIL=1
fi

if [ -f target/release/coders-bible-cli ]; then
    echo "  ✅ CLI release binary exists"
    ./target/release/coders-bible-cli stats --json >/dev/null && echo "  ✅ CLI stats command works" || echo "  ❌ CLI stats failed"
else
    echo "  ⚠️  CLI release binary not found (run: cargo build --release)"
fi

# ── Desktop Rust Backend ──────────────────────────────────────
echo ""
echo "▶ Desktop Rust Backend (lib only)"
cd "$(dirname "$0")/src-tauri"
# We can't compile the full Tauri binary without webkit2gtk-dev on Linux,
# but we can verify the library code compiles by checking syntax.
if cargo check --quiet 2>/dev/null; then
    echo "  ✅ Desktop crate compiles"
else
    echo "  ⚠️  Desktop crate check failed (may need: sudo apt install libwebkit2gtk-4.1-dev libsoup-3.0-dev)"
fi

# ── Frontend Assets ───────────────────────────────────────────
echo ""
echo "▶ Frontend Assets"
cd "$(dirname "$0")/src"
for f in index.html app.js style.css manifest.webmanifest service-worker.js; do
    if [ -f "$f" ]; then
        echo "  ✅ $f"
    else
        echo "  ❌ $f missing"
        FAIL=1
    fi
done

# ── Icons ─────────────────────────────────────────────────────
echo ""
echo "▶ Icons"
for s in 72 96 128 144 152 192 384 512; do
    if [ -f "icon-${s}.png" ]; then
        echo "  ✅ icon-${s}.png"
    else
        echo "  ❌ icon-${s}.png missing"
        FAIL=1
    fi
done

# ── VS Code Extension ─────────────────────────────────────────
echo ""
echo "▶ VS Code Extension"
VSCODE="$(dirname "$0")/../gravity-omega-v2/plugins/the-coders-bible/vscode-extension"
if [ -f "$VSCODE/coders-bible-2.1.0.vsix" ]; then
    echo "  ✅ coders-bible-2.1.0.vsix packaged"
else
    echo "  ⚠️  Extension .vsix not found"
fi

# ── CI/CD ─────────────────────────────────────────────────────
echo ""
echo "▶ CI/CD Workflows"
for wf in .github/workflows/release.yml ../coders-bible-cli/.github/workflows/cli.yml; do
    if [ -f "$(dirname "$0")/$wf" ]; then
        echo "  ✅ $wf"
    else
        echo "  ❌ $wf missing"
        FAIL=1
    fi
done

# ── Summary ───────────────────────────────────────────────────
echo ""
if [ $FAIL -eq 0 ]; then
    echo "🎉 All critical checks passed."
else
    echo "⚠️  Some checks failed. Review output above."
    exit 1
fi
