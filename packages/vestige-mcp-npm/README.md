# vestige-mcp-server

**v2.1.23** — Vestige MCP Server: local cognitive memory and optional Receipt Lock for MCP-compatible AI agents.

- **Memory:** FSRS-6 spaced repetition, prediction error gating, dreaming, 3D dashboard
- **Receipt Lock:** blocks "tests passed" / "build green" without command receipts (optional hooks)
- **Stats:** ~86K LOC Rust · 25 tools · 1,200+ tests · 22MB binary · 100% local

Homepage: https://samvallad33.github.io/vestige/ · Repo: https://github.com/samvallad33/vestige

## Installation

```bash
npm install -g vestige-mcp-server
```

This automatically downloads the correct binary for your platform (macOS, Linux, Windows) from GitHub releases.

Already installed? Update without copying release URLs:

```bash
vestige update
```

This refreshes the binaries only. Optional Claude Code Cognitive Sandwich
companion files are refreshed with `vestige update --sandwich-companion` or
`vestige sandwich install`.

### What gets installed

| Command | Description |
|---------|-------------|
| `vestige-mcp` | MCP server for local agent memory |
| `vestige` | CLI for stats, health checks, and maintenance |
| `vestige-restore` | Restore helper for backup recovery |

### Verify installation

```bash
vestige health
```

## Usage with MCP Clients

Vestige works with any client that can register a stdio MCP server.

**Claude Code**

```bash
claude mcp add vestige vestige-mcp -s user
```

**Codex**

```bash
codex mcp add vestige -- vestige-mcp
```

Then restart your MCP client.

## Optional Receipt Lock for Claude Code

Receipt Lock is part of Vestige's optional Cognitive Sandwich hook layer. Normal
MCP memory stays lightweight and local. If you want claim checking for summaries
like "tests passed" or "lint is clean," enable Sanhedrin and point it at any
OpenAI-compatible chat endpoint:

```bash
vestige sandwich install --enable-sanhedrin

vestige sandwich install \
  --enable-sanhedrin \
  --sanhedrin-endpoint=http://127.0.0.1:11434/v1/chat/completions \
  --sanhedrin-model=qwen2.5:14b
```

If a claim is missing command evidence, Vestige writes local receipts under
`~/.vestige/sanhedrin/` so the veto is inspectable instead of opaque.

## Usage with Claude Desktop

Add to your Claude Desktop configuration:

**macOS:** `~/Library/Application Support/Claude/claude_desktop_config.json`
**Windows:** `%APPDATA%\Claude\claude_desktop_config.json`

```json
{
  "mcpServers": {
    "vestige": {
      "command": "vestige-mcp"
    }
  }
}
```

## CLI Commands

```bash
vestige stats          # Memory statistics
vestige stats --states # Cognitive state distribution
vestige health         # System health check
vestige consolidate    # Run memory maintenance cycle
vestige update         # Update binaries
vestige update --sandwich-companion # Also refresh optional Claude Code files
vestige sandwich install # Manage optional Claude Code hook files
```

## Features

- **FSRS-6 Algorithm**: State-of-the-art spaced repetition for optimal memory retention
- **Receipt Lock**: Optional command-receipt checking for test/build/lint/typecheck claims
- **Dual-Strength Memory**: Bjork & Bjork (1992) - Storage + Retrieval strength model
- **Synaptic Tagging**: Memories become important retroactively (Frey & Morris 1997)
- **Semantic Search**: Local embeddings via nomic-embed-text-v1.5 (768 dimensions)
- **Local-First**: All data stays on your machine - no cloud, no API costs

## Storage & Memory

Vestige uses SQLite for storage. Your memories are stored on **disk**, not in RAM.

- **Database limit**: 216TB (SQLite theoretical max)
- **RAM usage**: ~64MB cache (configurable)
- **Typical usage**: 1 million memories ≈ 1-2GB on disk

You'll never run out of space. A heavy user creating 100 memories/day would use ~1.5GB after 10 years.

## Embeddings

On first use, Vestige downloads the nomic-embed-text-v1.5 model (~130MB). This is a one-time download and all subsequent operations are fully offline.

The model is stored in Vestige's OS cache directory, or you can set a global location:

```bash
export FASTEMBED_CACHE_PATH="$HOME/.fastembed_cache"
```

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `RUST_LOG` | Log verbosity + per-module filter | `info` |
| `FASTEMBED_CACHE_PATH` | Embeddings model cache override | OS cache dir |
| `VESTIGE_DATA_DIR` | Storage directory fallback; database lives at `<dir>/vestige.db` | OS data dir |
| `VESTIGE_DASHBOARD_PORT` | Dashboard port | `3927` |
| `VESTIGE_AUTH_TOKEN` | Bearer auth for dashboard + HTTP MCP | auto-generated |

Storage precedence is `--data-dir <path>`, then `VESTIGE_DATA_DIR`, then your OS's per-user data directory.

## Troubleshooting

### "Could not attach to MCP server vestige"

1. Verify binary exists: `which vestige-mcp`
2. Test directly: `vestige-mcp` (should wait for stdio input)
3. Check your MCP client's server logs.

### "vestige: command not found"

Reinstall the package:
```bash
npm install -g vestige-mcp-server
```

### Embeddings not downloading

The model downloads on first memory ingest or search operation. If your MCP
client cannot connect to the MCP server, no memory operations happen and no
model downloads.

Fix the MCP connection first, then the model will download automatically.

## Supported Platforms

| Platform | Architecture |
|----------|--------------|
| macOS | ARM64 (Apple Silicon), x86_64 (Intel) |
| Linux | x86_64 |
| Windows | x86_64 |

## License

AGPL-3.0-only
