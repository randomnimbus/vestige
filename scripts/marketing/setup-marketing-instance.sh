#!/usr/bin/env bash
# One-time setup: dedicated Vestige store + Claude Code MCP entry for marketing.
set -euo pipefail

MARKETING_DIR="${VESTIGE_MARKETING_DIR:-$HOME/.vestige-marketing}"
BIN_DIR="${HOME}/.local/bin"
WRAPPER="${BIN_DIR}/vestige-mcp-marketing"

echo "==> Marketing data dir: ${MARKETING_DIR}"
mkdir -p "${MARKETING_DIR}"

if ! command -v vestige-mcp >/dev/null 2>&1; then
  echo "Install Vestige first: npm install -g vestige-mcp-server@latest"
  exit 1
fi

mkdir -p "${BIN_DIR}"
cat > "${WRAPPER}" <<EOF
#!/usr/bin/env bash
export VESTIGE_DATA_DIR="${MARKETING_DIR}"
exec vestige-mcp "\$@"
EOF
chmod +x "${WRAPPER}"
echo "==> Wrapper: ${WRAPPER}"

if command -v claude >/dev/null 2>&1; then
  if claude mcp list 2>/dev/null | grep -q vestige-marketing; then
    echo "==> claude mcp: vestige-marketing already registered"
  else
    claude mcp add vestige-marketing "${WRAPPER}" -s user
    echo "==> Added: claude mcp add vestige-marketing ${WRAPPER} -s user"
  fi
else
  echo "==> Claude Code not found — register manually:"
  echo "    claude mcp add vestige-marketing ${WRAPPER} -s user"
fi

echo "==> Seeding baseline memories..."
"$(dirname "$0")/seed-baseline-memories.sh"

echo ""
echo "Done. Open Claude Code with MARKETING-CLAUDE.md:"
echo "  cp docs/marketing/growth-engine/MARKETING-CLAUDE.md ~/vestige-marketing-CLAUDE.md"
echo "  vestige health --data-dir ${MARKETING_DIR}"
