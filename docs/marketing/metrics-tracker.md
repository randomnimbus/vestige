# Vestige Growth Metrics Tracker

**North star:** weekly `vestige-mcp-server` npm installs + evidence of active MCP connections (issues, "it works" posts).

Update every **Monday**. Feed summary into marketing Vestige via `smart_ingest`.

## How to fetch numbers

```bash
# npm weekly downloads (approximate)
npm view vestige-mcp-server

# GitHub stars
gh api repos/samvallad33/vestige --jq .stargazers_count

# Optional: npm download chart
# https://www.npmjs.com/package/vestige-mcp-server
```

## Weekly log template

Copy a row per week:

| Week ending | npm downloads (total) | Stars | Stars Δ | Top channel | Top hook | Installs anecdote | Notes |
|-------------|----------------------|-------|---------|-------------|----------|-------------------|-------|
| 2026-06-02 | TBD | 542 | 0 | pre-launch | Receipt Lock / fake tests passed | setup complete | marketing instance seeded, ready for Wave A |
| 2026-06-09 | | | | | | | post Wave A week 1 |

## Per-post log template

| Date | Wave | Channel | Post URL | Engagement | Stars Δ (48h) | Objections | Action |
|------|------|---------|----------|------------|---------------|------------|--------|
| | A | HN | | | | | |

## Objection → content flywheel

When the same objection appears 3+ times, promote to permanent doc:

| Objection | Response doc |
|-----------|----------------|
| "Isn't this just RAG?" | [comparison.md](../comparison.md) |
| "Claude has memory now" | comparison.md + Receipt Lock section |
| "AGPL?" | README + HN FAQ in show-hn.md |
| "77K LOC over-engineered" | show-hn.md FAQ |
| "FSRS gimmick?" | [SCIENCE.md](../SCIENCE.md) |

## Agent ingest prompt (weekly)

```
smart_ingest: Vestige marketing week ending YYYY-MM-DD.
npm: X total (ΔY). Stars: N (ΔZ).
Best channel: [HN/Reddit/X].
Best hook: [phrase].
Top objection: [text].
Next week: [one action].
tags: marketing, metrics, vestige-launch
```

## Goals (first 8 weeks)

| Milestone | Target |
|-----------|--------|
| Wave A HN front page | 100+ points |
| Stars | 542 → 800+ |
| npm weekly downloads | 2× baseline |
| Registry listings | 5+ MCP directories |
