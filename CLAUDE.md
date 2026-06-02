# Vestige Agent Guidance

This file is intentionally safe for the public repository. It gives coding
agents project-specific context without relying on private local files,
personal operating notes, or mandatory background hooks.

## Project Shape

Vestige is a local-first MCP memory server written in Rust, with a SvelteKit
dashboard embedded into the release binary. The core product promise is:

- user-owned memory stored locally by default
- MCP-native integration with coding agents
- retrieval and memory lifecycle behavior informed by cognitive science
- explicit tools for search, review, suppression, purge, graph exploration,
  contradiction inspection, and maintenance

## Working Rules

- Prefer source evidence over memory. Use `rg`, tests, and nearby code before
  making claims about behavior. This public repo guidance does not override
  private user-level memory protocols loaded outside the repository.
- Keep release changes scoped. Do not rewrite unrelated modules during a
  version/tag cleanup unless the release gate requires it.
- Preserve local-first behavior. Heavy models, Sanhedrin-style verifier hooks,
  and preflight automation must remain optional.
- Treat deletion semantics carefully. `purge` must remove content and
  embeddings, while retaining only content-free audit tombstones.
- Treat exact lookup semantics carefully. Env vars, paths, UUIDs, quoted
  strings, and code identifiers should not be distorted by semantic expansion.

## Common Checks

Run the narrowest check that covers the change, then run the release gates
before tagging:

```sh
cargo test --workspace --no-fail-fast
cargo clippy --workspace -- -D warnings
pnpm --filter @vestige/dashboard check
pnpm --filter @vestige/dashboard build
```

For documentation-only changes, at minimum run:

```sh
git diff --check
```

## Documentation

- User setup: `README.md`
- Claude-specific templates: `docs/CLAUDE-SETUP.md`
- Storage and sync behavior: `docs/STORAGE.md`
- Cognitive Sandwich and optional verifier hooks: `docs/COGNITIVE_SANDWICH.md`
- Release history: `CHANGELOG.md`

## Public-Repo Hygiene

Do not commit private absolute paths, local agent memory paths, unpublished
planning files, real credentials, personal operating notes, or private repo
locations. Example environment variables in docs must be empty placeholders or
obviously fake examples.
