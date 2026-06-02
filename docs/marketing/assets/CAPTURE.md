# Demo GIF / Video Capture Guide

Record these on a machine with Vestige v2.1.23 installed and ~20 pre-loaded memories (see `docs/launch/demo-script.md` pre-load section).

## Prerequisites

```bash
npm install -g vestige-mcp-server@latest
vestige health
claude mcp add vestige vestige-mcp -s user
open http://localhost:3927/dashboard
```

## Assets to produce

| File | Duration | What to show |
|------|----------|----------------|
| `receipt-lock.gif` | 8–12s loop | Agent claims "tests passed" → Sanhedrin veto → `~/.vestige/sanhedrin/latest.html` receipt |
| `dashboard-dream.gif` | 10–15s loop | Graph view → trigger dream in Claude → purple dream mode, golden connection lines |
| `memory-born.gif` | 5–8s | Feed tab: `MemoryCreated` WebSocket event + new node burst on graph |
| `demo-full.mp4` | 60–90s | Full script: `docs/launch/demo-script.md` Version 2 (3-minute cut to 90s) |

## macOS capture (recommended)

```bash
# Screen recording → convert to GIF (install: brew install ffmpeg)
ffmpeg -f avfoundation -i "1" -t 12 -vf "fps=10,scale=1280:-1" -y /tmp/vestige-rec.mov
ffmpeg -i /tmp/vestige-rec.mov -vf "fps=8,scale=960:-1:flags=lanczos,split[s0][s1];[s0]palettegen[p];[s1][p]paletteuse" -loop 0 docs/marketing/assets/dashboard-dream.gif
```

## Static fallback

If GIFs are not ready for launch, export one PNG from the dashboard graph view:

```bash
# Browser screenshot → save as:
docs/marketing/assets/dashboard-static.png
```

Commit GIFs when ready; README and landing page reference these paths.
