# Vestige Marketing Growth Engine

Repeatable weekly loop: Vestige remembers what worked, Claude Code drafts what’s next, you approve and post manually.

## One-time setup

### 1. Dedicated marketing memory store

```bash
mkdir -p ~/.vestige-marketing
```

Add a **second** MCP server entry (do not mix with dev memory):

```bash
claude mcp add vestige-marketing vestige-mcp -s user \
  --env VESTIGE_DATA_DIR=$HOME/.vestige-marketing
```

If your client does not support env on `mcp add`, use a wrapper script:

```bash
# ~/bin/vestige-mcp-marketing
#!/bin/bash
export VESTIGE_DATA_DIR="$HOME/.vestige-marketing"
exec vestige-mcp "$@"
```

### 2. Copy marketing agent instructions

```bash
cp docs/marketing/growth-engine/MARKETING-CLAUDE.md ~/vestige-marketing-CLAUDE.md
```

In Claude Code for marketing sessions, include that file (or paste into project instructions).

### 3. Seed baseline memories

Open Claude Code with `vestige-marketing` connected and run:

```
Read docs/LAUNCH_STATS.md, docs/comparison.md, and docs/marketing/metrics-tracker.md.
smart_ingest each as separate marketing baseline memories with tags: marketing, baseline, vestige-launch.
```

## Weekly loop (≈2 hours)

| Step | Who | Action |
|------|-----|--------|
| Mon AM | You | Fill `metrics-tracker.md` row |
| Mon | Agent | `session_context` with query "vestige marketing launch" |
| Mon | Agent | Draft 1 long-form + 3–5 shorts from last week's `promote`d hooks |
| Mon | You | Edit and post manually (HN/Reddit/X/LinkedIn) |
| Fri | You | Log engagement URLs + numbers |
| Fri | Agent | `smart_ingest` weekly metrics + objections |
| Fri | Agent | `dream` on tag `vestige-launch` for next week's angles |

## Tool cheat sheet

| Goal | Tool |
|------|------|
| Load brand voice + past wins | `session_context` |
| Save post results | `smart_ingest` |
| Recall winning hooks | `search` / `deep_reference` |
| Retire dead angles | `suppress` |
| Boost viral hook | `memory` action=promote |
| Weekly strategy | `dream` |

## Dogfood story (meta-content)

> "I use Vestige to market Vestige — marketing memories live in a separate data dir, FSRS promotes hooks that converted, suppress kills angles that flopped."

Post this on X after Week 2 if metrics show engagement.

## Files

| File | Purpose |
|------|---------|
| [MARKETING-CLAUDE.md](MARKETING-CLAUDE.md) | Agent protocol |
| [../metrics-tracker.md](../metrics-tracker.md) | Weekly numbers |
| [../wave-a-launch.md](../wave-a-launch.md) | Receipt Lock execution |
| [../wave-b-launch.md](../wave-b-launch.md) | Memory wave execution |
| [../../launch/receipt-lock.md](../../launch/receipt-lock.md) | Wave A copy |
| [../../comparison.md](../../comparison.md) | Argument anchor |
