# Cognitive Sandwich

**Vestige's defense-in-depth safety architecture for Claude Code.**

The default Cognitive Sandwich installer only stages files and removes old v2.1.0 hook wiring. It activates no Claude Code hooks and makes no automatic model calls. Both the preflight layer and the Stop-hook layer are explicit opt-ins:

```
┌────────────────────────────────────────────────┐
│  🥪 TOP BREAD  — UserPromptSubmit hooks         │
│   • Vestige memory graph injection              │
│   • CWD / git / CI state injection              │
│   • Synthesis-protocol gate (decision-adjacent) │
│   • Lateral-thinker subconscious swarm          │
│   • Pulse daemon (background dream insights)    │
├────────────────────────────────────────────────┤
│  🥩 MEAT       — Claude Code reasons            │
├────────────────────────────────────────────────┤
│  🥪 OPTIONAL BOTTOM BREAD — Stop hooks          │
│   • Veto-detector / synthesis validator         │
│   • Sanhedrin Executioner verifier              │
└────────────────────────────────────────────────┘
```

Sanhedrin, preflight, and all Vestige Claude Code hooks are optional. The default installer wires none of them; it does not call Claude, start MLX, require a 19 GB model download, or require 20+ GB of RAM. Users who want preflight context can opt in with `--enable-preflight`. Users who want the post-response verifier can opt in with `--enable-sanhedrin` and point it at any OpenAI-compatible `/v1/chat/completions` endpoint. On Apple Silicon, an additional `--with-launchd` flag can auto-start the local MLX Qwen backend.

---

## How a single response flows through the Sandwich

1. **You type a prompt in Claude Code.**
2. **If explicitly enabled, UserPromptSubmit hooks fire in parallel** (none can block — all fail-open):
   - `load-all-memory.sh` (opt-in) — dumps every memory MD into context
   - `synthesis-preflight.sh` — POSTs your prompt to `vestige-mcp` `/api/deep_reference`, injects the trust-scored reasoning chain
   - `cwd-state-injector.sh` — captures git status, branch, open PRs/issues, modified files
   - `vestige-pulse-daemon.sh` — injects fresh Vestige dream insights from the past 20 min into the next prompt context
   - `preflight-swarm.sh` — spawns the `lateral-thinker` subagent in fresh context to surface cross-disciplinary structural parallels
3. **Claude reads the assembled context and generates a draft.**
4. **By default, no Vestige Stop hooks are installed.** If explicitly enabled, Stop hooks fire serially (any can VETO with `exit 2`, forcing a rewrite):
   - `veto-detector.sh` — fast regex against `veto`-tagged Vestige memories (~50ms)
   - `sanhedrin.sh` → `sanhedrin-local.py` — optional single-shot semantic verdict
   - `synthesis-stop-validator.sh` — regex against forbidden patterns (hedging, summary-instead-of-composition)
5. **If all enabled Stop hooks return `exit 0`, the response is delivered.**

---

## The Sanhedrin Executioner protocol

The Executioner extracts atomic claims from Claude's draft across 10 classes:

`TECHNICAL` · `BIOGRAPHICAL` · `FINANCIAL` · `ACHIEVEMENT` · `TIMELINE` · `QUANTITATIVE` · `ATTRIBUTION` · `CAUSAL` · `COMPARATIVE` · `EXISTENTIAL` · plus v2.1.0 additions: `VAGUE-QUANTIFIER` · `UNVERIFIED-POSITIVE`

For each claim, it checks Vestige's `deep_reference` for high-trust contradicting memories. Decision rules:

| Class | Rule |
|---|---|
| TECHNICAL / EXISTENTIAL / TIMELINE | VETO if memory trust > 0.55 directly contradicts |
| BIOGRAPHICAL / FINANCIAL / ACHIEVEMENT / ATTRIBUTION | VETO if contradicted OR if factual-shaped with zero supporting evidence (fail-closed) |
| **VAGUE-QUANTIFIER** | VETO on "a few wins / some prize money / most placed" without enumeration |
| **UNVERIFIED-POSITIVE** | VETO on specific named institutions/dates/employers not in evidence |

False-positive guards (added v2.1.0 after dogfood):
- Subject-equality gate (memory about Vestige codebase ≠ contradiction with external tools)
- Version-discriminator rule (M3 Max ≠ M5 Max; Qwen3.5 ≠ Qwen3.6)
- Agreement-is-not-contradiction (memory that AGREES with draft → PASS)
- Architecture-vs-component (overall architecture memory doesn't contradict component-level draft)
- Inference-verb ban (no `implies` / `suggests` / `must mean` in veto reasons)

---

## Installation

### One-liner

```bash
curl -fsSL https://raw.githubusercontent.com/samvallad33/vestige/v2.1.0/scripts/install-sandwich.sh | sh
```

### From a checkout

```bash
git clone https://github.com/samvallad33/vestige
cd vestige
./scripts/install-sandwich.sh           # add --force to overwrite existing hooks
./scripts/check-sandwich-prereqs.sh     # verify no Vestige hooks are wired by default
```

The default command does not activate any Claude Code hook. It removes old v2.1.0 Vestige hook wiring from `~/.claude/settings.json` while preserving unrelated user hooks.

### Optional Preflight

Preflight is a separate opt-in layer. It includes `preflight-swarm.sh`, which uses `claude -p --model claude-haiku-4-5-20251001`; it is not wired by default.

```bash
./scripts/install-sandwich.sh --enable-preflight
./scripts/check-sandwich-prereqs.sh --preflight
```

### Optional Sanhedrin

Sanhedrin is a separate opt-in layer.

```bash
# Wire the Sanhedrin Stop hook, using the default OpenAI-compatible endpoint.
./scripts/install-sandwich.sh --enable-sanhedrin

# Apple Silicon only, and only if the machine has enough memory:
./scripts/install-sandwich.sh --enable-sanhedrin --with-launchd

# x86 / Linux / Intel Mac: use any OpenAI-compatible endpoint.
./scripts/install-sandwich.sh \
  --enable-sanhedrin \
  --sanhedrin-endpoint=http://127.0.0.1:11434/v1/chat/completions \
  --sanhedrin-model=qwen2.5:14b
```

### Prerequisites

| Tool | Install |
|---|---|
| Python 3.10+ | typically preinstalled |
| `jq` | `brew install jq` |
| `vestige-mcp` | `cargo install vestige-mcp` |
| Claude Code | https://claude.ai/code |

Optional Apple Silicon local Sanhedrin backend:

| Tool | Install |
|---|---|
| macOS Apple Silicon (M1+) | required for MLX launchd only |
| `uv` | `brew install uv` |
| `mlx-lm` | `uv tool install mlx-lm` |
| `huggingface_hub[cli]` | `uv tool install 'huggingface_hub[cli]'` |
| Qwen3.6-35B-A3B-4bit | `hf download mlx-community/Qwen3.6-35B-A3B-4bit` (~19 GB) |

### What the installer does

1. Verifies prereqs (warnings for missing tools, fatal only on jq/python3).
2. Copies hooks to `~/.claude/hooks/`, agents to `~/.claude/agents/`.
3. Backs up existing `~/.claude/settings.json` to `.bak.pre-sandwich`, then removes old Vestige hook wiring from previous v2.1.0 installs.
4. With `--enable-preflight`, merges the UserPromptSubmit hooks block.
5. With `--enable-sanhedrin`, writes `~/.claude/hooks/vestige-sanhedrin.env` and merges a Sanhedrin-enabled Stop hooks block.
6. With `--enable-sanhedrin --with-launchd` on Apple Silicon, renders and loads `launchd/com.vestige.mlx-server.plist.template`.

### Uninstall

```bash
launchctl unload ~/Library/LaunchAgents/com.vestige.mlx-server.plist
rm ~/Library/LaunchAgents/com.vestige.mlx-server.plist
cp ~/.claude/settings.json.bak.pre-sandwich ~/.claude/settings.json
# Hook files in ~/.claude/hooks/ can be deleted manually.
```

---

## Performance notes

Optional local MLX backend on M3 Max 16-core (400 GB/s memory bandwidth):
- Sanhedrin verdict: 5–15 seconds end-to-end (single deep_reference + single Qwen call)
- mlx_lm.server token generation: ~82 tok/s
- mlx_lm.server peak resident memory: ~19.7 GB
- Cold model load: ~5 seconds

On M3 Max 14-core or M2/M1 Max: closer to 3–7s prompt processing, ~50–60 tok/s generation.

---

## Configuration

| Env var | Default | Effect |
|---|---|---|
| `VESTIGE_SANHEDRIN_ENABLED` | `0` | Set to `1` to enable the optional Sanhedrin Stop hook |
| `VESTIGE_SWARM_ENABLED` | `1` | Set to `0` to disable preflight lateral-thinker swarm |
| `VESTIGE_DASHBOARD_PORT` | `3927` | Vestige MCP HTTP API port used by hooks |
| `VESTIGE_SANHEDRIN_ENDPOINT` | `http://127.0.0.1:8080/v1/chat/completions` | OpenAI-compatible chat completions endpoint for Sanhedrin |
| `VESTIGE_SANHEDRIN_MODEL` | `mlx-community/Qwen3.6-35B-A3B-4bit` | Model name sent to the Sanhedrin endpoint |
| `MLX_ENDPOINT` / `VESTIGE_SANDWICH_MODEL` | legacy aliases | Backward-compatible names still read by the bridge |
| `VESTIGE_MEMORY_DIR` | (auto) | Override per-user Claude memory dir |

---

## Architecture provenance

The Cognitive Sandwich originated April 2026 as a defense against the AIMO3 36/50 failure mode — Claude retrieving relevant memories but summarizing them instead of composing them into recommendations. The pre-cognitive layer enforces composition; the post-cognitive layer catches contradictions before they ship.

Full architecture memory: search Vestige for `god-tier-plan` or `cognitive-sandwich` tags after install.

---

## Linux / Intel Mac / x86

The base hook harness runs on x86. The launchd MLX helper is macOS-arm64-only.

On Linux, Windows under WSL, or Intel Mac:
- Run `scripts/install-sandwich.sh` normally to stage files and remove old Vestige hook wiring. No hooks are activated.
- If you want Sanhedrin, run an OpenAI-compatible endpoint such as vLLM, Ollama, llama.cpp server, or a remote MLX/vLLM box.
- Install with `--enable-sanhedrin --sanhedrin-endpoint=<url> --sanhedrin-model=<model>`.
- If the endpoint is unreachable, Sanhedrin fails open and does not block Claude Code.
