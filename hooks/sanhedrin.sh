#!/bin/bash
# sanhedrin.sh — Stop hook (Post-Cognitive Sanhedrin / Full Agent-Type Guillotine)
#
# Spawns the Executioner subagent (Haiku 4.5, fresh context, Vestige MCP
# tools) to run mcp__vestige__deep_reference 8-stage contradiction analysis
# on the last assistant draft. If any technical claim contradicts a
# high-trust memory, exit 2 with the veto reason — forces Main Claude to
# rewrite.
#
# Runs AFTER veto-detector.sh (fast regex against veto-tagged memories).
# Sanhedrin is the deeper semantic check: it reads the draft as a real
# reasoning agent, extracts claims, runs deep_reference on each.
#
# Architecture:
#   Main Claude finishes draft → Stop hook chain fires →
#   veto-detector.sh (50ms regex, may block) →
#   sanhedrin.sh (2-8s Haiku subagent, may block) →
#   synthesis-stop-validator.sh (existing regex hedge check, may block)
#
# Opt-in: set VESTIGE_SANHEDRIN_ENABLED=1 in parent shell, or install with
# scripts/install-sandwich.sh --enable-sanhedrin.
# Re-entrancy lock: VESTIGE_EXECUTIONER_ACTIVE=1 inside the subagent.
#
# Ship date 2026-04-20.

set -u

# === OPT-IN GATE ===
# Sanhedrin is heavyweight: the default local backend is a ~19 GB model and
# needs roughly 20+ GB of free RAM. Keep it disabled unless the user explicitly
# opts in. The installer writes this env file only for --enable-sanhedrin.
SANHEDRIN_ENV="${VESTIGE_SANHEDRIN_ENV:-$HOME/.claude/hooks/vestige-sanhedrin.env}"
if [ -f "$SANHEDRIN_ENV" ]; then
  set +u
  set -a
  # shellcheck disable=SC1090
  . "$SANHEDRIN_ENV" 2>/dev/null || {
    set +a
    set -u
    exit 0
  }
  set +a
  set -u
fi

case "${VESTIGE_SANHEDRIN_ENABLED:-0}" in
  1|true|TRUE|yes|YES|on|ON) ;;
  *) exit 0 ;;
esac

# === RE-ENTRANCY GUARD ===
# The Executioner's own Stop hook will fire when it returns — prevent
# recursive spawns that would fork-bomb the quota.
if [ "${VESTIGE_EXECUTIONER_ACTIVE:-0}" = "1" ]; then
  exit 0
fi

# === READ STOP HOOK INPUT ===
INPUT="$(cat)"
TRANSCRIPT_PATH="$(printf '%s' "$INPUT" | /usr/bin/python3 -c 'import sys,json;d=json.load(sys.stdin);print(d.get("transcript_path",""))' 2>/dev/null || printf '')"

if [ -z "$TRANSCRIPT_PATH" ] || [ ! -f "$TRANSCRIPT_PATH" ]; then
  exit 0
fi

# === EXTRACT LAST ASSISTANT DRAFT ===
# Read the transcript JSONL, pull the last assistant message text.
export TRANSCRIPT_PATH
DRAFT_SCRIPT="$(mktemp -t vestige-sanhedrin-draft.XXXXXX)"
trap 'rm -f "$DRAFT_SCRIPT"' EXIT

cat > "$DRAFT_SCRIPT" <<'DRAFT_PYEOF'
import json, os, sys

transcript = os.environ.get("TRANSCRIPT_PATH", "")
last_assistant = ""

try:
    with open(transcript) as f:
        for line in f:
            line = line.strip()
            if not line:
                continue
            try:
                obj = json.loads(line)
            except Exception:
                continue
            role = obj.get("role") or obj.get("type", "")
            content = obj.get("message", {}).get("content", obj.get("content", ""))
            text = ""
            if isinstance(content, list):
                for block in content:
                    if isinstance(block, dict) and block.get("type") == "text":
                        text += block.get("text", "") + "\n"
            elif isinstance(content, str):
                text = content
            if role == "assistant":
                last_assistant = text
except Exception:
    sys.exit(0)

# Print nothing if no draft or draft too short to contain a technical claim
stripped = last_assistant.strip()
if not stripped or len(stripped) < 100:
    sys.exit(0)

# Gate: only check drafts that contain technical indicators
has_code = "`" in stripped or "```" in stripped
has_cmd = any(kw in stripped.lower() for kw in ["install", "run ", "use ", "call ", "invoke", "execute"])
has_path = "/" in stripped and any(ext in stripped for ext in [".rs", ".ts", ".py", ".sh", ".md", ".json"])

if not (has_code or has_cmd or has_path):
    sys.exit(0)

# Truncate to 4000 chars to keep Haiku prompt bounded
if len(stripped) > 4000:
    stripped = stripped[:4000] + "... [truncated]"

print(stripped)
DRAFT_PYEOF

DRAFT="$(/usr/bin/python3 "$DRAFT_SCRIPT" 2>/dev/null || printf '')"

if [ -z "$DRAFT" ]; then
  exit 0
fi

# === VERIFY local executioner bridge available ===
# 2026-04-25: switched from Haiku 4.5 subagent to an OpenAI-compatible
# local/remote endpoint. On Apple Silicon the optional launchd path starts
# mlx_lm.server; on x86 users can point VESTIGE_SANHEDRIN_ENDPOINT at vLLM,
# Ollama, llama.cpp, or any compatible /v1/chat/completions endpoint.
# Fail-open if the endpoint is unreachable.
BRIDGE="$HOME/.claude/hooks/sanhedrin-local.py"
if [ ! -x "$BRIDGE" ] && [ ! -f "$BRIDGE" ]; then
  exit 0
fi

# === SPAWN LOCAL EXECUTIONER (background with timeout) ===
OUTPUT_FILE="$(mktemp -t vestige-sanhedrin-out.XXXXXX)"
trap 'rm -f "$DRAFT_SCRIPT" "$OUTPUT_FILE"' EXIT

(
  printf '%s\n' "$DRAFT" | /usr/bin/python3 "$BRIDGE" > "$OUTPUT_FILE" 2>/dev/null
) &

EXEC_PID=$!

# === TIMEOUT GUARD (60 seconds) ===
# Local Qwen3.6-35B-A3B on M5/M3 Max typically returns in 5-15s for the
# single-shot judgment. 60s ceiling preserves the existing settings.json
# Stop hook timeout (70s) and gives headroom for cold model load if
# launchd just restarted. Bridge fail-opens internally if mlx-server is
# unreachable, so timeout-kill here is the secondary safety net.
WAITED=0
while [ "$WAITED" -lt 60 ]; do
  if ! /bin/kill -0 "$EXEC_PID" 2>/dev/null; then
    break
  fi
  sleep 1
  WAITED=$((WAITED + 1))
done
if /bin/kill -0 "$EXEC_PID" 2>/dev/null; then
  /bin/kill "$EXEC_PID" 2>/dev/null
  wait "$EXEC_PID" 2>/dev/null
  exit 0
fi
wait "$EXEC_PID" 2>/dev/null

EXECUTIONER_OUTPUT="$(cat "$OUTPUT_FILE" 2>/dev/null || printf '')"

# === PARSE VERDICT ===
TRIMMED="$(printf '%s' "$EXECUTIONER_OUTPUT" | /usr/bin/awk 'NF {print; exit}' | /usr/bin/awk '{$1=$1;print}')"

if [ -z "$TRIMMED" ]; then
  exit 0
fi

# "yes" verdict - draft is clean, allow stop
case "$TRIMMED" in
  yes|YES|Yes|yes.|Yes.)
    exit 0
    ;;
esac

# "no - <reason>" or "no: <reason>" verdict - block the stop, force rewrite
# Documented spec is `no - [Sanhedrin Veto] [CLASS]: <reason>` (hyphen-space).
# Legacy `no: <reason>` also accepted for backward compat.
case "$TRIMMED" in
  no\ -*|NO\ -*|No\ -*|no:*|NO:*|No:*)
    case "$TRIMMED" in
      no\ -*|NO\ -*|No\ -*)
        REASON="${TRIMMED#* - }"
        ;;
      *)
        REASON="${TRIMMED#*:}"
        ;;
    esac
    REASON="$(printf '%s' "$REASON" | /usr/bin/awk '{$1=$1;print}')"

    cat >&2 <<SANHEDRIN_MSG
[SANHEDRIN VETO - Post-Cognitive Executioner (LOCAL) rejected draft]

$REASON

The Executioner (Sanhedrin endpoint, fresh context, fed Vestige
deep_reference evidence over HTTP) judged your draft and
found a contradiction against a high-trust memory.

You may NOT stop. Rewrite WITHOUT the contradicted claim. Use
mcp__vestige__deep_reference to inspect the cited memory and cite the
correct replacement pattern from its \`recommended\` field.

Bridge script:
~/.claude/hooks/sanhedrin-local.py
SANHEDRIN_MSG
    exit 2
    ;;
esac

# Unparseable verdict — fail open (do not block on Executioner errors)
exit 0
