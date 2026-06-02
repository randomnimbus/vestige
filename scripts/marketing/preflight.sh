#!/usr/bin/env bash
# Pre-launch checks before Wave A posts.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "${ROOT}"

FAIL=0
pass() { echo "OK  $1"; }
fail() { echo "FAIL $1"; FAIL=1; }

echo "=== Vestige Launch Preflight ==="

command -v vestige-mcp >/dev/null && pass "vestige-mcp on PATH" || fail "vestige-mcp not found"
command -v npm >/dev/null && pass "npm available" || fail "npm missing"

VER="$(vestige-mcp --version 2>/dev/null || true)"
[[ "${VER}" == *"2.1."* ]] && pass "version ${VER}" || fail "unexpected version: ${VER}"

vestige health >/dev/null 2>&1 && pass "vestige health" || fail "vestige health failed"

[[ -f docs/LAUNCH_STATS.md ]] && pass "LAUNCH_STATS.md" || fail "missing LAUNCH_STATS.md"
[[ -f docs/launch/receipt-lock.md ]] && pass "receipt-lock.md" || fail "missing receipt-lock.md"
[[ -f docs/website/index.html ]] && pass "landing page source" || fail "missing website"
[[ -f .github/workflows/pages.yml ]] && pass "pages workflow" || fail "missing pages workflow"

if curl -sf --max-time 5 "https://samvallad33.github.io/vestige/" >/dev/null 2>&1; then
  pass "GitHub Pages live"
else
  echo "WARN GitHub Pages not live yet — push main and enable Pages → GitHub Actions"
fi

MARKETING_DIR="${HOME}/.vestige-marketing"
if [[ -d "${MARKETING_DIR}" ]]; then
  pass "marketing data dir exists"
else
  echo "WARN run scripts/marketing/setup-marketing-instance.sh"
fi

echo ""
if [[ "${FAIL}" -eq 0 ]]; then
  echo "Preflight PASSED — ready for Wave A"
  exit 0
else
  echo "Preflight FAILED — fix items above"
  exit 1
fi
