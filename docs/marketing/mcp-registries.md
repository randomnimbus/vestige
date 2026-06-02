# MCP Registry & Directory Submissions

Passive install channel — update listings whenever v2.1.x ships. Check off as you submit.

## Submission packet (reuse everywhere)

| Field | Value |
|-------|-------|
| Name | Vestige |
| Slug | `io.github.samvallad33/vestige` (npm `mcpName`) |
| Description | Local cognitive memory for MCP agents — FSRS-6 spaced repetition, prediction error gating, memory dreaming, 3D dashboard, optional Receipt Lock for agent verification claims. |
| Install | `npm install -g vestige-mcp-server@latest` then `claude mcp add vestige vestige-mcp -s user` |
| Repo | https://github.com/samvallad33/vestige |
| Homepage | https://samvallad33.github.io/vestige/ |
| License | AGPL-3.0-only |
| Transport | stdio (default); HTTP opt-in |
| Version | 2.1.23 |
| Tags | memory, mcp, claude, cursor, local-first, fsrs, neuroscience, rust |

## Registries

| Directory | URL | Status | Notes |
|-----------|-----|--------|-------|
| Glama | https://glama.ai/mcp/servers | [ ] Submit / refresh | Ownership metadata in repo (`cd496e5`) |
| mcp.so | https://mcp.so | [ ] Submit | Use submission packet |
| Smithery | https://smithery.ai | [ ] Submit | npm package + stdio command |
| PulseMCP | https://www.pulsemcp.com | [ ] Submit | |
| Awesome MCP Servers | https://github.com/punkpeye/awesome-mcp-servers | [ ] PR | Add under Memory / Knowledge |
| modelcontextprotocol/servers | https://github.com/modelcontextprotocol/servers | [ ] PR if accepted | Follow their CONTRIBUTING |
| Cursor directory | docs/integrations/cursor.md | [x] Doc exists | Link from Cursor forum / Discord |
| VS Code marketplace | N/A for MCP stdio | [ ] N/A | Use integrations/vscode.md in posts |

## Awesome-MCP PR snippet

```markdown
### Vestige
- **Description:** Local cognitive memory — FSRS-6 decay, dreaming, contradiction tools, optional Receipt Lock
- **Install:** `npm install -g vestige-mcp-server@latest`
- **Command:** `vestige-mcp`
- **Repo:** https://github.com/samvallad33/vestige
```

## After each listing goes live

```bash
# Ingest into marketing Vestige
smart_ingest: "Listed Vestige on [REGISTRY] at [URL]. Version 2.1.23."
tags: marketing, registry, vestige-launch
```

## Editor-specific posts (optional)

| Community | Action |
|-----------|--------|
| Cursor Discord #showcase | Link comparison.md + 30s dashboard GIF |
| Claude Code GitHub discussions | Receipt Lock angle + install |
| r/mcp | Neutral "new server" post after Wave B |
