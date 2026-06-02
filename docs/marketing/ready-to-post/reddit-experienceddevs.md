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
