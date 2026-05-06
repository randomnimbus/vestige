# Vestige State And Plan

This document is a public, sanitized replacement for an older internal planning
snapshot. It intentionally omits private local paths, personal operating
context, unpublished roadmap notes, and private repository locations.

For current user-facing release information, use:

- `README.md`
- `CHANGELOG.md`
- `docs/STORAGE.md`
- `docs/COGNITIVE_SANDWICH.md`
- `docs/CLAUDE-SETUP.md`

## Current Release Shape

Vestige v2.1.2 is the "Honest Memory" release. Its public scope is:

- concrete literal search for quoted strings, env vars, UUIDs, paths, and code
  identifiers
- irreversible purge semantics with content-free deletion tombstones
- first-class contradiction inspection through the MCP `contradictions` tool
- the `vestige update` CLI flow for binary and Cognitive Sandwich updates
- dense dream connection persistence fixes
- embedding-model upgrade repair during consolidation
- an opt-in `/dashboard/waitlist` preview for Vestige Pro early access

The release keeps the local-first baseline intact. Heavy model hooks, local
verifier models, and preflight automation remain optional.

## Release Gates

Before tagging a release, run:

```sh
cargo test --workspace --no-fail-fast
cargo clippy --workspace -- -D warnings
pnpm --filter @vestige/dashboard check
pnpm --filter @vestige/dashboard build
git diff --check
```

For dashboard route changes, rebuild and stage `apps/dashboard/build/` so the
embedded static assets match `apps/dashboard/src/`.

## Product Principles

- Exact things should stay exact. Literal identifiers should not lose to
  semantic expansion.
- Forgetting should be honest. A hard purge should remove content, embeddings,
  graph edges, and derived references while retaining only non-content proof
  that deletion happened.
- Contradictions should be visible. Trust-weighted disagreement should be
  inspectable directly instead of hidden inside a broader reasoning tool.
- Installation should remain boring. Users should not need a large local model
  or background hook system just to use memory.
- Pro features should add managed convenience without weakening local-first
  ownership.

## Public Architecture Summary

Vestige is organized as:

- `crates/vestige-core`: storage, search, embeddings, memory lifecycle, FSRS,
  graph, dream, and cognitive modules
- `crates/vestige-mcp`: MCP server, CLI, dashboard backend, tools, update flow
- `apps/dashboard`: SvelteKit dashboard source
- `packages/vestige-mcp-npm`: npm wrapper for the MCP binary
- `packages/vestige-init`: installer helper
- `docs`: user and integration documentation

## v2.1.2 Implementation Notes

Concrete search is implemented in the MCP `search` tool and core SQLite
storage. Literal-looking queries use a keyword path instead of HyDE expansion,
semantic fusion, FSRS reweighting, retrieval competition, and spreading
activation.

Purge is implemented transactionally in storage and surfaced through the MCP
`memory` tool. `memory(action="purge", confirm=true)` is the explicit hard
delete path. `delete` remains a backwards-compatible alias.

Contradictions are exposed as a first-class MCP tool and reuse the same trust
and topic-overlap logic used by the deeper reference pipeline.

The waitlist preview is a dashboard route. Its capture and support endpoints
are controlled by opt-in public dashboard environment variables. If unset, the
page does not silently capture private signup data.

## 15. Autopilot Rationale

The backend event bus exists so dashboard and MCP activity can be observed by
the cognitive engine without making user-facing agent hooks mandatory. Any
autonomous behavior should be conservative, rate-limited, and local-first.

Autopilot-style routing should never require a remote model, a heavy local
model, or a Claude hook to make normal memory useful. It should only connect
already-emitted Vestige events to existing cognitive modules when that improves
maintenance, retrieval quality, or dashboard fidelity without surprising the
user.
