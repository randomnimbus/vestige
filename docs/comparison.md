# Vestige vs Mem0 vs RAG vs Native AI Memory

Canonical comparison for launch posts and arguments. Grounded in [SCIENCE.md](SCIENCE.md) and [LAUNCH_STATS.md](LAUNCH_STATS.md).

## One-line thesis

**RAG is retrieval. Native memory is a black box. Mem0 is a strong cloud memory API. Vestige is a local cognitive system that forgets, strengthens, dreams, and can block unverified agent claims.**

## Comparison table

| Capability | RAG / vector DB | Native AI memory (Claude, ChatGPT) | Mem0 | Vestige |
|------------|-----------------|-------------------------------------|------|---------|
| **Runs local** | Often cloud embeddings | Cloud only | Cloud API (local option limited) | **100% local** default |
| **You own the data** | Your infra | Vendor | Vendor / API | **SQLite on your disk** |
| **Forgetting curve** | None — equal weight forever | Opaque | Categories + metadata | **FSRS-6** power-law decay |
| **Duplicate handling** | Manual | Opaque | Some dedup | **Prediction Error Gating** on ingest |
| **Retrieval strengthens memory** | No | Unknown | Partial | **Testing Effect** on every search |
| **Offline consolidation** | No | No | No | **`dream`** — replay + connect |
| **Contradiction awareness** | Returns both chunks | No | Some products | **`deep_reference` / `contradictions`** |
| **Active suppression** | Delete only | No | Delete | **`suppress`** — inhibited, not erased |
| **Agent overclaim guard** | No | No | No | **Receipt Lock** (optional Sanhedrin hooks) |
| **Visualization** | None | None | Dashboard (cloud) | **3D graph** + WebSocket events |
| **Protocol** | Custom | Proprietary | API + MCP | **MCP** (25 tools) |
| **License** | Varies | Proprietary | Apache / commercial | **AGPL-3.0** (local use = free) |

## When to use what

### Use RAG when

- You have a fixed document corpus (PDFs, wiki, codebase index).
- You need one-shot Q&A over static content.
- You do not need memory lifecycle or session continuity.

### Use Mem0 when

- You want a hosted memory API with minimal setup.
- Team sync and cloud dashboards are acceptable.
- You do not need FSRS decay or local-only air-gapped deploy.

### Use native Claude/ChatGPT memory when

- Casual personal context is enough.
- You do not need inspectable storage, decay curves, or contradiction tooling.

### Use Vestige when

- You run **Claude Code, Cursor, Codex, or any MCP client** daily.
- Context bloat from "remember everything" hurts retrieval quality.
- **Contradicting memories** have burned you (config changed, lib upgraded).
- You want **Receipt Lock** so agents cannot fake "tests passed."
- **Privacy / air-gapped** matters — embeddings run locally via ONNX.

## Honest limitations (Vestige)

- **AGPL-3.0**: hosting as a service without source disclosure is not allowed.
- **First-run download**: ~130MB embedding model (then offline).
- **Receipt Lock** requires optional Claude Code Cognitive Sandwich hooks + a verifier endpoint for Sanhedrin.
- **Neuroscience modules** mix faithful implementations and engineering heuristics — see [SCIENCE.md](SCIENCE.md) for citations vs approximations.
- **Solo project**: no enterprise SLA; GitHub issues are the support channel.

## Receipt Lock (Vestige-only)

Coding agents often end sessions with:

> "All tests passed. Build is green. Ready to merge."

Receipt Lock checks those **operational claims** against structured command receipts from the transcript. No matching successful receipt → claim blocked, local veto receipt written under `~/.vestige/sanhedrin/`.

```bash
vestige sandwich install --enable-sanhedrin
```

Details: [README Receipt Lock section](../README.md#receipt-lock).

## Install

```bash
npm install -g vestige-mcp-server@latest
claude mcp add vestige vestige-mcp -s user
```

Full stats: [LAUNCH_STATS.md](LAUNCH_STATS.md) · Repo: https://github.com/samvallad33/vestige
