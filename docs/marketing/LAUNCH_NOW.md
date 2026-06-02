# LAUNCH NOW — Wave A (Receipt Lock)

**Date started:** 2026-06-02  
**Copy-paste from:** [ready-to-post/](ready-to-post/)

## 5-minute sequence

1. Run preflight:
   ```bash
   ./scripts/marketing/preflight.sh
   ```

2. **Hacker News** → https://news.ycombinator.com/submit
   - URL: `https://github.com/samvallad33/vestige`
   - Title: paste `ready-to-post/hn-title.txt`
   - Post link, then immediately paste `ready-to-post/hn-first-comment.txt` as first comment

3. **X** — paste `ready-to-post/x-thread.txt` (one tweet per numbered block)

4. **r/ExperiencedDevs** — title from `reddit-experienceddevs-title.txt`, body from `reddit-experienceddevs.md`

5. **r/programming** — same body + line: `License: AGPL-3.0. v2.1.23. ~86K LOC, 25 tools, 22MB binary.`

6. Log URLs in [metrics-tracker.md](metrics-tracker.md)

## After posting (30 min SLA on comments)

```bash
vestige ingest "Wave A posted YYYY-MM-DD on HN Reddit X. Hook: agent fake tests passed. Log URLs in metrics-tracker." \
  --data-dir ~/.vestige-marketing --tags marketing,wave-a,vestige-launch
```

## 48h later → Wave B

[wave-b-launch.md](wave-b-launch.md) + [show-hn.md](../launch/show-hn.md) memory angle
