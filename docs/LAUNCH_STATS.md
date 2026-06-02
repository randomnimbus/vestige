# Vestige Launch Stats (Single Source of Truth)

**Last verified:** 2026-06-02  
**Use this file** when updating README, launch posts, npm README, and landing page. Do not invent numbers elsewhere.

## Release

| Field | Value |
|-------|-------|
| Version | **v2.1.23** ("Receipt Lock Hardening") |
| npm package | `vestige-mcp-server@latest` |
| Install | `npm install -g vestige-mcp-server@latest` |
| MCP connect (Claude Code) | `claude mcp add vestige vestige-mcp -s user` |
| Optional Receipt Lock | `vestige sandwich install --enable-sanhedrin` |
| License | AGPL-3.0-only |
| Repo | https://github.com/samvallad33/vestige |
| Homepage (marketing) | https://samvallad33.github.io/vestige/ (GitHub Pages) |

## Author

| Field | Value |
|-------|-------|
| Name | Sam Valladares |
| Age | **22** (solo developer) |
| GitHub stars (2026-06-02) | **542** |
| Forks | **55** |

## Codebase (run to refresh)

```bash
# Rust LOC (crates + tests)
find crates tests -name '*.rs' | xargs wc -l | tail -1

# MCP tool count (must match server assertion)
rg 'name: "' crates/vestige-mcp/src/server.rs | wc -l

# Tests
cargo test --workspace --no-fail-fast 2>&1 | tail -3
```

| Metric | Current value | Notes |
|--------|---------------|-------|
| Rust LOC | **~86,000** | `crates/` + `tests/` `.rs` files |
| MCP tools | **25** | Verified in `server.rs` (`tools.len() == 25`) |
| Cognitive modules | **30** | Per README architecture |
| Rust tests | **1,200+** | CHANGELOG v2.1.0: 1,229 passing; re-run before major launch |
| Dashboard tests | **171** | Vitest in `apps/dashboard` |
| Release binary | **~22MB** | Single binary, embedded SvelteKit dashboard |
| Embedding model | Nomic Embed Text v1.5 (~130MB first-run download) |

## Install (canonical — no `sudo mv`)

The npm package registers global bins via `postinstall`. **Do not** tell users to `sudo mv vestige-mcp` unless manual binary install failed.

```bash
npm install -g vestige-mcp-server@latest
vestige health
claude mcp add vestige vestige-mcp -s user
```

If `vestige-mcp` is not on PATH after install:

```bash
npm prefix -g   # e.g. /usr/local or ~/.npm-global
# Ensure that path/bin is in your shell PATH
```

Manual binary placement (optional):

```bash
vestige update --install-dir /usr/local/bin
```

## Messaging guardrails

- Lead Wave A with **Receipt Lock** (agents overclaim "tests passed").
- Close Wave B with **cognitive memory** (FSRS-6, dreaming, 3D dashboard).
- Never: "revolutionary", "game-changer", "AI-powered", competitor bashing.
- Always: honest neuroscience (faithful implementations vs engineering heuristics).

## North-star metrics

Track weekly (see `docs/marketing/metrics-tracker.md`):

1. **npm downloads** (`npm view vestige-mcp-server` / npmjs.com stats)
2. **GitHub stars delta**
3. **Inbound issues/DMs** mentioning install
4. **Referral source** (HN, Reddit, X, registry)
