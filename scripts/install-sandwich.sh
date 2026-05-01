#!/usr/bin/env bash
# install-sandwich.sh — One-command installer for the Vestige Cognitive Sandwich.
#
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/samvallad33/vestige/v2.1.0/scripts/install-sandwich.sh | sh
#   # or, from a checkout:
#   ./scripts/install-sandwich.sh [--force] [--enable-preflight] [--enable-sanhedrin] [--with-launchd] [--include-memory-loader]
#   ./scripts/install-sandwich.sh --enable-sanhedrin --sanhedrin-endpoint=http://127.0.0.1:11434/v1/chat/completions --sanhedrin-model=qwen2.5:14b
#
# What it does:
#   1. Verifies required local tools
#   2. Stages ~/.claude/hooks/ and ~/.claude/agents/
#   3. Copies sanitized hooks + agents
#   4. Removes old Vestige hook wiring from ~/.claude/settings.json by default
#   5. Optionally enables preflight hooks and/or Sanhedrin. Only with --with-launchd on Apple Silicon,
#      auto-starts mlx_lm.server with Qwen3.6-35B-A3B

set -euo pipefail

VERSION="${VESTIGE_SANDWICH_VERSION:-v2.1.0}"
REPO="samvallad33/vestige"
MODEL_ID="${VESTIGE_SANHEDRIN_MODEL:-${VESTIGE_SANDWICH_MODEL:-mlx-community/Qwen3.6-35B-A3B-4bit}}"
DASHBOARD_PORT="${VESTIGE_DASHBOARD_PORT:-3927}"
SANHEDRIN_ENDPOINT="${VESTIGE_SANHEDRIN_ENDPOINT:-${MLX_ENDPOINT:-http://127.0.0.1:8080/v1/chat/completions}}"
SANHEDRIN_ENDPOINT="${SANHEDRIN_ENDPOINT%/}"
SANHEDRIN_MODELS_URL="${SANHEDRIN_ENDPOINT%/chat/completions}/models"

HOOKS_DIR="$HOME/.claude/hooks"
AGENTS_DIR="$HOME/.claude/agents"
LAUNCHD_DIR="$HOME/Library/LaunchAgents"
SETTINGS="$HOME/.claude/settings.json"

FORCE=0
ENABLE_PREFLIGHT=0
ENABLE_SANHEDRIN=0
WITH_LAUNCHD=0
INCLUDE_MEMORY_LOADER=0
SRC=""

for arg in "$@"; do
  case "$arg" in
    --force) FORCE=1 ;;
    --enable-preflight) ENABLE_PREFLIGHT=1 ;;
    --enable-sandwich) ENABLE_PREFLIGHT=1; ENABLE_SANHEDRIN=1 ;;
    --enable-sanhedrin) ENABLE_SANHEDRIN=1 ;;
    --with-launchd) WITH_LAUNCHD=1 ;;
    --no-launchd) WITH_LAUNCHD=0 ;;
    --include-memory-loader) INCLUDE_MEMORY_LOADER=1 ;;
    --sanhedrin-endpoint=*|--endpoint=*)
      SANHEDRIN_ENDPOINT="${arg#*=}"
      SANHEDRIN_ENDPOINT="${SANHEDRIN_ENDPOINT%/}"
      SANHEDRIN_MODELS_URL="${SANHEDRIN_ENDPOINT%/chat/completions}/models"
      ;;
    --sanhedrin-model=*|--model=*)
      MODEL_ID="${arg#*=}"
      ;;
    --src=*) SRC="${arg#--src=}" ;;
    -h|--help)
      sed -n '2,24p' "$0"
      exit 0
      ;;
  esac
done

if [ "$WITH_LAUNCHD" -eq 1 ] && [ "$ENABLE_SANHEDRIN" -eq 0 ]; then
  ENABLE_SANHEDRIN=1
fi

say()  { printf '\033[1;36m[sandwich]\033[0m %s\n' "$*"; }
warn() { printf '\033[1;33m[sandwich]\033[0m %s\n' "$*" >&2; }
die()  { printf '\033[1;31m[sandwich]\033[0m %s\n' "$*" >&2; exit 1; }

# --- Platform check ---
OS_NAME="$(uname -s)"
ARCH_NAME="$(uname -m)"
say "platform: $OS_NAME $ARCH_NAME"
if [ "$ENABLE_SANHEDRIN" -eq 1 ] && [ "$WITH_LAUNCHD" -eq 0 ]; then
  say "Sanhedrin enabled without launchd; using OpenAI-compatible endpoint: $SANHEDRIN_ENDPOINT"
fi
if [ "$WITH_LAUNCHD" -eq 1 ] && { [ "$OS_NAME" != "Darwin" ] || [ "$ARCH_NAME" != "arm64" ]; }; then
  warn "--with-launchd is Apple Silicon only; skipping local MLX autostart on $OS_NAME $ARCH_NAME"
  warn "Sanhedrin can still run on x86 via --sanhedrin-endpoint or VESTIGE_SANHEDRIN_ENDPOINT."
  WITH_LAUNCHD=0
fi

# --- Prereqs (warnings only, install proceeds) ---
command -v jq      >/dev/null || die "jq required: brew install jq"
command -v python3 >/dev/null || die "python3 required (3.10+)"
if [ "$ENABLE_PREFLIGHT" -eq 1 ]; then
  command -v claude  >/dev/null || warn "'claude' CLI not found — preflight-swarm.sh will fail open."
  command -v vestige-mcp >/dev/null || warn "'vestige-mcp' not found — Vestige preflight hooks will fail open."
fi
if [ "$WITH_LAUNCHD" -eq 1 ]; then
  command -v uv      >/dev/null || warn "'uv' not found — install with: brew install uv"
  command -v mlx_lm.server >/dev/null || warn "mlx-lm not installed — run: uv tool install mlx-lm"
  command -v hf      >/dev/null || warn "'hf' not found — run: uv tool install 'huggingface_hub[cli]'"
fi

# --- Resolve source: local checkout or release tarball ---
if [ -n "$SRC" ]; then
  SCRIPT_DIR="$SRC"
elif [ -f "$(dirname "$0")/../hooks/sanhedrin.sh" ]; then
  SCRIPT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
  say "Using local checkout: $SCRIPT_DIR"
else
  TMPDIR="$(mktemp -d)"
  trap 'rm -rf "$TMPDIR"' EXIT
  say "Fetching Vestige Sandwich $VERSION..."
  curl -fsSL "https://github.com/$REPO/archive/refs/tags/$VERSION.tar.gz" \
    | tar xz -C "$TMPDIR"
  SCRIPT_DIR="$(ls -d "$TMPDIR"/vestige-*)"
fi

[ -d "$SCRIPT_DIR/hooks" ] || die "hooks/ not found in $SCRIPT_DIR — wrong source?"

# --- Stage directories ---
mkdir -p "$HOOKS_DIR" "$AGENTS_DIR"
if [ "$WITH_LAUNCHD" -eq 1 ]; then
  mkdir -p "$LAUNCHD_DIR"
fi

# v2.1.0 originally installed the MLX server as part of the default path.
# Default reinstalls now retire that job; users can restore it with --with-launchd.
if [ "$WITH_LAUNCHD" -eq 0 ] && [ "$OS_NAME" = "Darwin" ]; then
  LEGACY_PLIST="$LAUNCHD_DIR/com.vestige.mlx-server.plist"
  if [ -f "$LEGACY_PLIST" ]; then
    launchctl unload "$LEGACY_PLIST" 2>/dev/null || true
    rm -f "$LEGACY_PLIST"
    say "removed old Sanhedrin launchd job (use --with-launchd to opt back in)"
  fi
fi

# --- Copy hooks ---
copied=0; skipped=0
for f in "$SCRIPT_DIR/hooks"/*.sh "$SCRIPT_DIR/hooks"/*.py; do
  [ -f "$f" ] || continue
  base="$(basename "$f")"
  # load-all-memory.sh dumps every memory MD — opt-in only
  if [ "$base" = "load-all-memory.sh" ] && [ "$INCLUDE_MEMORY_LOADER" -eq 0 ]; then
    say "skip $base (use --include-memory-loader to install)"
    continue
  fi
  if [ -e "$HOOKS_DIR/$base" ] && [ "$FORCE" -eq 0 ]; then
    skipped=$((skipped + 1))
    continue
  fi
  install -m 0755 "$f" "$HOOKS_DIR/$base"
  copied=$((copied + 1))
done
say "hooks: $copied installed, $skipped skipped (use --force to overwrite)"

# --- Copy agents ---
for f in "$SCRIPT_DIR/agents"/*.md; do
  [ -f "$f" ] || continue
  base="$(basename "$f")"
  if [ -e "$AGENTS_DIR/$base" ] && [ "$FORCE" -eq 0 ]; then
    continue
  fi
  install -m 0644 "$f" "$AGENTS_DIR/$base"
done
say "agents installed to $AGENTS_DIR"

# --- Persist optional Sanhedrin env ---
quote_env() {
  printf "'%s'" "$(printf '%s' "$1" | sed "s/'/'\\\\''/g")"
}

if [ "$ENABLE_SANHEDRIN" -eq 1 ]; then
  SANHEDRIN_ENV="$HOOKS_DIR/vestige-sanhedrin.env"
  {
    printf 'VESTIGE_SANHEDRIN_ENABLED=1\n'
    printf 'VESTIGE_SANHEDRIN_ENDPOINT=%s\n' "$(quote_env "$SANHEDRIN_ENDPOINT")"
    printf 'VESTIGE_SANHEDRIN_MODEL=%s\n' "$(quote_env "$MODEL_ID")"
    printf 'VESTIGE_DASHBOARD_PORT=%s\n' "$(quote_env "$DASHBOARD_PORT")"
  } > "$SANHEDRIN_ENV"
  chmod 0600 "$SANHEDRIN_ENV"
  say "Sanhedrin opt-in config written to $SANHEDRIN_ENV"
fi

# --- Render launchd plist (Apple Silicon opt-in only) ---
if [ "$WITH_LAUNCHD" -eq 1 ]; then
  PLIST="$LAUNCHD_DIR/com.vestige.mlx-server.plist"
  TEMPLATE="$SCRIPT_DIR/launchd/com.vestige.mlx-server.plist.template"
  [ -f "$TEMPLATE" ] || die "launchd template missing: $TEMPLATE"
  sed -e "s|__HOME__|$HOME|g" -e "s|__MODEL__|$MODEL_ID|g" "$TEMPLATE" > "$PLIST"
  launchctl unload "$PLIST" 2>/dev/null || true
  launchctl load "$PLIST"
  say "launchd loaded: com.vestige.mlx-server (model: $MODEL_ID)"
fi

# --- Merge hooks fragment into settings.json ---
[ -f "$SETTINGS" ] || echo '{}' > "$SETTINGS"
if [ -f "$HOME/.claude/settings.json.bak.pre-sandwich" ]; then
  say "settings.json backup already exists at .bak.pre-sandwich — not overwriting"
else
  cp "$SETTINGS" "$HOME/.claude/settings.json.bak.pre-sandwich"
fi
TMP_MERGE="$(mktemp)"
PREFLIGHT_FRAGMENT="$SCRIPT_DIR/hooks/settings.fragment.json"
SANHEDRIN_FRAGMENT="$SCRIPT_DIR/hooks/settings.fragment.json"
if [ "$ENABLE_PREFLIGHT" -eq 1 ]; then
  PREFLIGHT_FRAGMENT="$SCRIPT_DIR/hooks/settings.preflight.fragment.json"
fi
if [ "$ENABLE_SANHEDRIN" -eq 1 ]; then
  SANHEDRIN_FRAGMENT="$SCRIPT_DIR/hooks/settings.sanhedrin.fragment.json"
fi
jq -s '
  def is_vestige_hook:
    (.command? // "") as $cmd
    | [
        "synthesis-preflight.sh",
        "cwd-state-injector.sh",
        "vestige-pulse-daemon.sh",
        "preflight-swarm.sh",
        "load-all-memory.sh",
        "veto-detector.sh",
        "sanhedrin.sh",
        "synthesis-stop-validator.sh",
        "synthesis-gate.sh"
      ] | any(. as $needle | $cmd | contains($needle));

  def scrub_vestige_hooks:
    .hooks.UserPromptSubmit = (
      (.hooks.UserPromptSubmit // [])
      | map(.hooks = ((.hooks // []) | map(select((is_vestige_hook | not)))))
      | map(select(((.hooks // []) | length) > 0))
    )
    | if ((.hooks.UserPromptSubmit // []) | length) == 0 then del(.hooks.UserPromptSubmit) else . end
    | .hooks.Stop = (
      (.hooks.Stop // [])
      | map(.hooks = ((.hooks // []) | map(select((is_vestige_hook | not)))))
      | map(select(((.hooks // []) | length) > 0))
    )
    | if ((.hooks.Stop // []) | length) == 0 then del(.hooks.Stop) else . end
    | if ((.hooks // {}) | length) == 0 then del(.hooks) else . end;

  (.[0] | scrub_vestige_hooks) * .[1] * .[2]
' "$SETTINGS" "$PREFLIGHT_FRAGMENT" "$SANHEDRIN_FRAGMENT" > "$TMP_MERGE"
mv "$TMP_MERGE" "$SETTINGS"
if [ "$ENABLE_PREFLIGHT" -eq 1 ] || [ "$ENABLE_SANHEDRIN" -eq 1 ]; then
  enabled_layers=""
  [ "$ENABLE_PREFLIGHT" -eq 1 ] && enabled_layers="${enabled_layers} preflight"
  [ "$ENABLE_SANHEDRIN" -eq 1 ] && enabled_layers="${enabled_layers} sanhedrin"
  say "merged optional hook layer(s) into $SETTINGS:${enabled_layers} (backup at .bak.pre-sandwich)"
else
  say "removed Vestige hook wiring from $SETTINGS; default install activates no Claude Code hooks (backup at .bak.pre-sandwich)"
fi

# --- Next steps ---
cat <<EOF

  ┌──────────────────────────────────────────────────────────────┐
  │  Cognitive Sandwich files installed. No hooks enabled by default. │
  └──────────────────────────────────────────────────────────────┘

  Next steps:
    1. Restart Claude Code if you enabled optional hooks.
       Default installs activate no Vestige Claude Code hooks and make no model calls.
    2. Verify the install:
         vestige health                 # if vestige CLI installed
         curl http://127.0.0.1:$DASHBOARD_PORT/api/health
         scripts/check-sandwich-prereqs.sh   # from a checkout
    3. Optional hook layers:
         ./scripts/install-sandwich.sh --enable-preflight
         ./scripts/install-sandwich.sh --enable-sanhedrin --sanhedrin-endpoint=$SANHEDRIN_ENDPOINT --sanhedrin-model=$MODEL_ID
       On Apple Silicon with >20 GB free RAM, add --with-launchd to auto-start
       the local MLX Qwen server. On x86, point --sanhedrin-endpoint at vLLM,
       Ollama, llama.cpp, or another OpenAI-compatible /v1/chat/completions URL.

  To uninstall:
    launchctl unload $LAUNCHD_DIR/com.vestige.mlx-server.plist 2>/dev/null || true
    rm -f $LAUNCHD_DIR/com.vestige.mlx-server.plist
    cp $HOME/.claude/settings.json.bak.pre-sandwich $HOME/.claude/settings.json

EOF
