# Wave A Launch — Receipt Lock (v2.1.23)

Primary viral hook. Post **before** the memory/science Show HN wave. Stats: [LAUNCH_STATS.md](../LAUNCH_STATS.md).

---

## Hacker News — Show HN

### Title (≤80 chars)

```
Show HN: Vestige – blocks coding agents from claiming "tests passed" without receipts
```

### First comment (body)

```
Hi HN,

Your coding agent probably ends sessions with something like "all tests passed" or
"the build is green." I kept trusting that — until it wasn't true.

I built Receipt Lock in Vestige (an MCP memory server I maintain). Before operational
claims become part of the final answer, Vestige checks them against structured
command receipts from the current transcript. No matching successful receipt → the
claim can be blocked and a local veto receipt is written (JSON + HTML under
~/.vestige/sanhedrin/).

**What it is:** Optional Claude Code Cognitive Sandwich hooks + local MCP server.
Not cloud. Not "trust me bro" logging — inspectable receipts on disk.

**Install (memory server — required base):**
npm install -g vestige-mcp-server@latest
claude mcp add vestige vestige-mcp -s user

**Enable Receipt Lock (optional):**
vestige sandwich install --enable-sanhedrin

Sanhedrin verifier can point at any OpenAI-compatible endpoint (Ollama, MLX, hosted API).

**And it also does real memory:** FSRS-6 spaced repetition, prediction error gating,
memory dreaming, 3D dashboard at localhost:3927. ~86K LOC Rust, 25 MCP tools, 1,200+
tests, 22MB binary. 100% local after first embedding download.

I'm 22, solo, AGPL-3.0. Repo: https://github.com/samvallad33/vestige
Comparison: https://github.com/samvallad33/vestige/blob/main/docs/comparison.md

Happy to discuss false positive tuning, Sanhedrin presets, or why receipts beat vibes.
```

---

## r/ExperiencedDevs

### Title

```
My coding agent kept saying "tests passed" when they hadn't. I added a receipt check before the summary ships.
```

### Body

```markdown
**TL;DR:** Vestige Receipt Lock checks operational claims ("tests passed", "build green", "lint clean") against structured command receipts from the transcript. No receipt → block + local veto artifact.

**The failure mode:** Agent runs partial checks, or hallucinates a green ending. You merge. CI breaks. You've seen this.

**The fix:** Optional hooks (`vestige sandwich install --enable-sanhedrin`) + MCP memory server. When the model tries to assert verification without evidence, Vestige can veto and write `~/.vestige/sanhedrin/latest.html` so you can inspect what happened.

**Not a replacement for CI.** It's a last-mile guard on *agent-authored* summaries in Claude Code.

**Stack:** Rust, local, MCP. Same project also does FSRS-6 cognitive memory (decay, dreaming, contradiction tools) — I'll post that angle separately if people want the science side.

```bash
npm install -g vestige-mcp-server@latest
claude mcp add vestige vestige-mcp -s user
vestige sandwich install --enable-sanhedrin
```

GitHub: https://github.com/samvallad33/vestige

What false positives are you seeing with agent verification claims? Curious if this matches your workflow.
```

---

## r/programming

### Title

```
Open-source guard: coding agents can't claim "tests passed" without command receipts (local MCP, Rust)
```

### Body — use r/ExperiencedDevs body; add:

```markdown
License: AGPL-3.0. v2.1.23. Stats: ~86K LOC, 25 tools, 22MB binary.
```

---

## X / Twitter thread (8 posts)

1. Your coding agent ends with "tests passed." Did it run tests? Or did it summarize hope?

2. I ship Receipt Lock in Vestige — checks operational claims against command receipts from the transcript.

3. No matching successful receipt → claim blocked. Local veto receipt: `~/.vestige/sanhedrin/latest.html`

4. Optional hooks. Local MCP server. Not cloud analytics.

5. ```bash
   npm i -g vestige-mcp-server@latest
   claude mcp add vestige vestige-mcp -s user
   vestige sandwich install --enable-sanhedrin
   ```

6. Same binary also does FSRS-6 memory — decay, dreaming, 3D brain viz. Thread on that tomorrow.

7. 22yo solo dev. AGPL. https://github.com/samvallad33/vestige

8. What's the worst "green build" lie your agent told you? Reply — building the FAQ from real stories.

---

## Lobste.rs

### Title

```
Vestige Receipt Lock: local MCP guard against unverified "tests passed" agent claims
```

### Tags

`rust` `programming` `security`

### Body

Use HN first comment (shorter). Link comparison.md.

---

## Engagement playbook (Wave A)

| Window | Action |
|--------|--------|
| 0–3h | Reply every comment within 30 min |
| Tone | Technical, humble, no "revolutionary" |
| Competitors | Acknowledge Mem0/Cursor memory; don't bash |
| CTA | Install + link comparison.md |
| Next | Schedule Wave B 48h after Wave A peaks |

### DO NOT

- "Game-changer" / "AI-powered" / "paradigm shift"
- Disparage Mem0 or Claude native memory
- Promise Receipt Lock replaces CI

---

## Timing

- **HN / Lobsters:** Tuesday or Wednesday, 8–10 AM US Eastern
- **Reddit:** Same day, +1–2h after HN
- **X:** Pin thread during HN peak
