#!/usr/bin/env bash
# Seed marketing Vestige with launch facts (separate from dev memory).
set -euo pipefail

MARKETING_DIR="${VESTIGE_MARKETING_DIR:-$HOME/.vestige-marketing}"
TAGS="marketing,baseline,vestige-launch"

ingest() {
  vestige ingest "$1" --data-dir "${MARKETING_DIR}" --tags "${TAGS}" --node-type note
}

echo "Seeding into ${MARKETING_DIR}..."

ingest "Vestige v2.1.23 launch stats: ~86K LOC Rust, 25 MCP tools, 30 cognitive modules, 1200+ tests, 22MB binary, AGPL-3.0, npm vestige-mcp-server@latest, homepage samvallad33.github.io/vestige"

ingest "Wave A hook Receipt Lock: block operational claims like tests passed or build green unless matching command receipts exist. Optional vestige sandwich install --enable-sanhedrin. Veto receipts at ~/.vestige/sanhedrin/"

ingest "Wave B product: FSRS-6 spaced repetition memory, prediction error gating, memory dreaming, 3D dashboard localhost:3927, deep_reference contradictions, 100 percent local after embedding download"

ingest "Canonical install: npm install -g vestige-mcp-server@latest && vestige health && claude mcp add vestige vestige-mcp -s user. Do NOT tell users sudo mv unless manual binary install failed."

ingest "Messaging guardrails: no revolutionary game-changer AI-powered. Acknowledge Mem0 RAG native Claude memory honestly. Lead pain first tool second. Author Sam Valladares age 22 solo."

ingest "North star metric: weekly npm installs vestige-mcp-server and active MCP connections not stars alone. Track in docs/marketing/metrics-tracker.md"

ingest "Comparison anchor docs/comparison.md: RAG is retrieval, Vestige is cognitive lifecycle with forgetting consolidation Receipt Lock. Mem0 is cloud API Vestige is local AGPL."

vestige stats --data-dir "${MARKETING_DIR}" 2>/dev/null || true
echo "Baseline seed complete."
