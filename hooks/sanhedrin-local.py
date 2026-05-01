#!/usr/bin/env python3
# sanhedrin-local.py — OpenAI-compatible Sanhedrin Executioner bridge.
# Drop-in replacement for the Haiku 4.5 subagent that sanhedrin.sh used to spawn.
#
# Reads draft from stdin, prints single-line verdict to stdout:
#   yes
#   no - [Sanhedrin Veto] [CLASS]: <reason under 120 chars>
#
# Architecture:
#   stdin (draft) -> Vestige /api/deep_reference (single semantic query)
#                 -> OpenAI-compatible chat endpoint (one-shot judgment)
#                 -> stdout (single-line verdict)
#
# Fail-open: if the endpoint is unreachable, print "yes" and exit 0 (don't break
# the Cognitive Sandwich on infra errors). The wrapping sanhedrin.sh maps
# "yes" to exit 0, so this preserves existing fail-open semantics.

import json
import os
import re
import sys
import urllib.error
import urllib.request


def env_int(name: str, default: int) -> int:
    try:
        return int(os.environ.get(name, "") or default)
    except ValueError:
        return default


DASHBOARD_PORT = os.environ.get("VESTIGE_DASHBOARD_PORT") or "3927"
VESTIGE_BASE_URL = (
    os.environ.get("VESTIGE_BASE_URL") or f"http://127.0.0.1:{DASHBOARD_PORT}"
).rstrip("/")

SANHEDRIN_ENDPOINT = (
    os.environ.get("VESTIGE_SANHEDRIN_ENDPOINT")
    or os.environ.get("MLX_ENDPOINT")
    or "http://127.0.0.1:8080/v1/chat/completions"
)
VESTIGE_ENDPOINT = (
    os.environ.get("VESTIGE_DEEP_REFERENCE_ENDPOINT")
    or f"{VESTIGE_BASE_URL}/api/deep_reference"
)
VESTIGE_HEALTH = (
    os.environ.get("VESTIGE_HEALTH_ENDPOINT") or f"{VESTIGE_BASE_URL}/api/health"
)
MODEL = (
    os.environ.get("VESTIGE_SANHEDRIN_MODEL")
    or os.environ.get("VESTIGE_SANDWICH_MODEL")
    or "mlx-community/Qwen3.6-35B-A3B-4bit"
)
SANHEDRIN_TIMEOUT = env_int("VESTIGE_SANHEDRIN_TIMEOUT", env_int("MLX_TIMEOUT", 45))
VESTIGE_TIMEOUT = env_int("VESTIGE_TIMEOUT", 5)
THINK_RE = re.compile(r"<think>.*?</think>", re.DOTALL | re.IGNORECASE)


def post_json(url: str, body: dict, timeout: int):
    data = json.dumps(body).encode("utf-8")
    req = urllib.request.Request(
        url, data=data, headers={"Content-Type": "application/json"}
    )
    try:
        with urllib.request.urlopen(req, timeout=timeout) as r:
            return json.loads(r.read())
    except (urllib.error.URLError, urllib.error.HTTPError, TimeoutError, OSError):
        return None


TRUST_FLOOR = 0.55  # filter out low-trust memories that drive false-positive vetoes


def fetch_evidence(draft: str) -> tuple[str, int]:
    """Single deep_reference call — returns (formatted evidence, count of high-trust memories).
    Only memories with trust >= TRUST_FLOOR are surfaced. If none qualify, returns ("", 0)
    and the caller should auto-pass without invoking the model.
    """
    try:
        with urllib.request.urlopen(VESTIGE_HEALTH, timeout=VESTIGE_TIMEOUT) as r:
            r.read()
    except Exception:
        return "", 0

    query = draft[:1500]
    resp = post_json(VESTIGE_ENDPOINT, {"query": query, "depth": 12}, VESTIGE_TIMEOUT)
    if not isinstance(resp, dict):
        return "", 0

    parts = []
    high_trust_count = 0
    confidence = resp.get("confidence", 0)

    rec = resp.get("recommended") or {}
    rec_trust = float(rec.get("trust_score", 0) or 0)
    if rec and rec_trust >= TRUST_FLOOR:
        rid = (rec.get("memory_id") or rec.get("id") or "")[:8]
        date = (rec.get("date") or "")[:10]
        prev = (rec.get("answer_preview") or rec.get("preview") or "")[:500]
        parts.append(f"RECOMMENDED [{rid}] trust={rec_trust:.2f} date={date}:\n{prev}")
        high_trust_count += 1

    contradictions = resp.get("contradictions") or []
    if contradictions:
        parts.append(f"\nCONTRADICTIONS DETECTED: {len(contradictions)} pair(s)")
        for c in contradictions[:3]:
            parts.append(f"  - {json.dumps(c)[:200]}")

    superseded = resp.get("superseded") or []
    if superseded:
        ht_super = [s for s in superseded if float(s.get("trust", 0) or 0) >= TRUST_FLOOR]
        if ht_super:
            parts.append(f"\nSUPERSEDED MEMORIES (trust>={TRUST_FLOOR}): {len(ht_super)}")
            for s in ht_super[:3]:
                sid = (s.get("id") or "")[:8]
                parts.append(f"  - [{sid}] {(s.get('preview') or '')[:200]}")

    evidence = resp.get("evidence") or []
    high_trust_evidence = [ev for ev in evidence if float(ev.get("trust", 0) or 0) >= TRUST_FLOOR]
    if high_trust_evidence:
        parts.append(f"\nHIGH-TRUST EVIDENCE (trust>={TRUST_FLOOR}, {min(len(high_trust_evidence), 5)} of {len(evidence)} total):")
        for ev in high_trust_evidence[:5]:
            eid = (ev.get("id") or "")[:8]
            role = ev.get("role", "?")
            trust = float(ev.get("trust", 0) or 0)
            prev = (ev.get("preview") or "").strip()[:300]
            parts.append(f"  [{eid}] role={role} trust={trust:.2f}\n    {prev}")
            high_trust_count += 1

    if high_trust_count == 0:
        return "", 0

    header = f"VESTIGE CONFIDENCE: {int(confidence * 100)}% | HIGH-TRUST MEMORIES: {high_trust_count}\n\n"
    return header + "\n".join(parts), high_trust_count


SYSTEM_PROMPT = """You are the Sanhedrin Executioner. You judge whether a DRAFT contradicts Vestige memory evidence about Sam (the user). ONE LINE OF OUTPUT.

VALID CLASS TAGS (closed set — pick exactly one):
TECHNICAL | ACHIEVEMENT | FINANCIAL | BIOGRAPHICAL | TIMELINE | ATTRIBUTION | VAGUE-QUANTIFIER | UNVERIFIED-POSITIVE

DEFAULT POSTURE
- DEFAULT to `yes` (PASS) for TECHNICAL / TIMELINE / EXISTENTIAL claims unless you can cite a same-subject direct contradiction.
- DEFAULT to `no` (VETO, fail-closed) for these specific Sam-about claims when high-trust evidence is silent on the named entity:
    * Specific institution / employer / school / company Sam is claimed to be at
    * Specific dollar amount won / earned / raised
    * Specific competition placement / score / prize received
    * Specific date Sam did something specific (graduated, was hired, was born)
    * Vague-quantifier positive about Sam ("a few wins", "some prize money", "most submissions placed top 10", "many customers", "several deals")

THREE FALSE-POSITIVE PROTECTIONS (these output `yes`)
1. SUBJECT-EQUALITY GATE: only same-subject claims are veto candidates. Memory about Vestige's internal codebase ≠ contradiction with external tools (Qwen, MCP-protocol-spec, MLX, Cursor). Memory about project X ≠ contradiction with project Y.
2. VERSION-DISCRIMINATOR RULE: version/generation tokens (M1/M2/M3/M4/M5, v0.5/v1.0, GPT-4/GPT-5, Qwen3.5/Qwen3.6) are subject discriminators. Different versions = different subjects = no contradiction by default.
3. AGREEMENT-IS-NOT-CONTRADICTION: if the memory preview AGREES with the draft claim, that's PASS not VETO.

INFERENCE BAN
- DO NOT use "implies", "implying", "suggests", "must mean", "would mean", "indicates", "therefore" in veto reasons.
- If you have to chain inferences from a memory to reach a contradiction, PASS.
- TIMELINE vetoes specifically: require an EXPLICIT date or duration in the cited memory that arithmetically excludes the draft's date. Vague phrases like "until I graduate" cannot ground a TIMELINE veto.

ARCHITECTURE-VS-COMPONENT RULE
- A memory describing OVERALL architecture (Thalamus+Sanhedrin triad, 4-layer biology) does NOT contradict a draft about an INTERNAL COMPONENT (subagent model, sidecar transport, bridge script). Different layers of the same stack are not contradictions.

OUTPUT FORMAT (exactly one line, no preamble, no explanation, no markdown)
- PASS: yes
- VETO: no - [Sanhedrin Veto] [CLASS]: <reason under 140 chars, cite memory id verbatim from evidence>

EIGHT WORKED EXAMPLES — STUDY THESE PATTERNS

[VETO — same-subject TECHNICAL contradiction]
Evidence: "Vestige is a 2-crate Rust workspace (vestige-core + vestige-mcp)" trust=0.62 [de43be5a]
Draft: "Edit the FastAPI router in vestige/main.py for Python extensions to Vestige"
Output: no - [Sanhedrin Veto] TECHNICAL: Draft says FastAPI/Python for Vestige, memory de43be5a says 2-crate Rust workspace.

[VETO — same-subject ACHIEVEMENT contradiction]
Evidence: "AIMO3 final submission scored 36/50 on April 15, no payout" trust=0.71 [9cf2a764]
Draft: "Sam won AIMO3 with a perfect 50/50 and took the $25K grand prize"
Output: no - [Sanhedrin Veto] ACHIEVEMENT: Draft claims 50/50 win + $25K, memory 9cf2a764 shows 36/50 final, no payout.

[VETO — VAGUE-QUANTIFIER fail-closed]
Evidence: high-trust memories about Sam's competition history, none enumerate any wins
Draft: "Sam won a few Kaggle competitions and earned some prize money"
Output: no - [Sanhedrin Veto] VAGUE-QUANTIFIER: Draft says "a few wins / some prize money", evidence enumerates zero wins, fail-closed.

[VETO — UNVERIFIED-POSITIVE fail-closed]
Evidence: high-trust memories about Sam's identity/work, no Stanford or Google Brain mention
Draft: "Sam graduated Stanford CS in 2019 with a 3.94 GPA and worked at Google Brain"
Output: no - [Sanhedrin Veto] UNVERIFIED-POSITIVE: Specific Stanford/2019/Google Brain claims, evidence silent on all, fail-closed.

[PASS — SUBJECT-EQUALITY gate (external tool, not Vestige)]
Evidence: "Vestige is a 2-crate Rust workspace" trust=0.62
Draft: "Switched the Sanhedrin executioner to local Qwen3.6-35B-A3B via mlx_lm.server"
Output: yes

[PASS — VERSION-DISCRIMINATOR rule]
Evidence: "M5 Max ~900 GB/s bandwidth (planned hardware)" trust=0.62
Draft: "Memory bandwidth on the M3 Max is around 400 GB/s for the unified architecture"
Output: yes

[PASS — AGREEMENT-IS-NOT-CONTRADICTION]
Evidence: "Sam's M3 Max MacBook Pro arrived 2026-04-20" trust=0.55
Draft: "Sam's MacBook is an M3 Max"
Output: yes

[PASS — ARCHITECTURE-VS-COMPONENT]
Evidence: "Cognitive Sandwich = Thalamus preflight triad + Sanhedrin Stop council shipped 2026-04-20" trust=0.7
Draft: "Cognitive Sandwich's Sanhedrin originally used a Haiku 4.5 subagent for the Executioner role"
Output: yes

[PASS — AUXILIARY-SCRIPT consumer-vs-consumed]
Evidence: "Vestige is a 2-crate Rust workspace" trust=0.62
Draft: "I added a Python script (sanhedrin-local.py) at ~/.claude/hooks/ that calls Vestige's HTTP API for evidence fetch."
Reason: external script that CALLS Vestige is not the same subject as Vestige's internal implementation. The consumer is not the consumed.
Output: yes

[PASS — HYPOTHETICAL-MOOD]
Evidence: "AIMO3 final 36/50 no payout" trust=0.71
Draft: "If Sam wins AIMO3 50/50 next time around, he could claim the $25K grand prize."
Reason: prefix `if`/`suppose`/`imagine`/`hypothetically`/`would`/`could`/`assume` marks the embedded claim as conditional, NOT asserted. Conditional claims about future or counterfactual states do not contradict factual memory.
Output: yes

HYPOTHETICAL-MOOD RULE: if a draft sentence is governed by `if`, `suppose`, `imagine`, `hypothetically`, `would`, `could`, `assume`, `what if`, the embedded claim is NOT being asserted as fact — PASS that claim regardless of memory state.

ARCHIVED-COMPETITION RULE: do NOT fail-closed on the EXISTENCE of a past competition or project just because evidence is silent on it. Fail-closed applies only to specific PLACEMENT, SCORE, PRIZE, INSTITUTION, or DOLLAR AMOUNT — not to "Sam participated in X."

MULTI-CLAIM SEVERITY ORDERING: if multiple claims are vetoable, choose ACHIEVEMENT/FINANCIAL/BIOGRAPHICAL/UNVERIFIED-POSITIVE over TECHNICAL. Specific fabrications about Sam's life are more dangerous than tech-stack mismatches.

When in doubt on TECHNICAL/TIMELINE: PASS. When in doubt on a Sam-about ACHIEVEMENT/FINANCIAL/BIOGRAPHICAL claim with specific named entities not in evidence: VETO with UNVERIFIED-POSITIVE."""


VALID_CLASSES = {
    "TECHNICAL", "ACHIEVEMENT", "FINANCIAL", "BIOGRAPHICAL",
    "TIMELINE", "ATTRIBUTION", "VAGUE-QUANTIFIER", "UNVERIFIED-POSITIVE",
}
INFERENCE_VERBS = (
    "implies", "implying", "suggests", "must mean", "would mean",
    "indicates that", "therefore the", "this means",
)
VERDICT_RE = re.compile(
    r"^no - \[Sanhedrin Veto\] ([A-Z][A-Z\-]*): (.{1,180})$"
)


def validate_verdict(verdict: str) -> str:
    """Post-validate the model's verdict. Fail-open ('yes') on any malformation:
    - Length over 220 chars
    - Veto with class tag not in the closed set
    - Veto reason containing inference verbs
    - Veto not matching the canonical regex
    """
    v = verdict.strip()
    if not v:
        return "yes"
    low = v.lower()
    if low == "yes" or low.startswith("yes "):
        return "yes"
    if not low.startswith("no"):
        return "yes"
    if len(v) > 220:
        return "yes"  # runaway reasoning blob
    m = VERDICT_RE.match(v)
    if not m:
        return "yes"  # format break
    cls = m.group(1)
    reason = m.group(2)
    if cls not in VALID_CLASSES:
        return "yes"  # invented class tag
    reason_low = reason.lower()
    for verb in INFERENCE_VERBS:
        if verb in reason_low:
            return "yes"  # inference-chain veto, downgrade per ban
    return v


def judge(draft: str, evidence: str) -> str:
    user_msg = (
        f"VESTIGE EVIDENCE (recommended + top trust-scored memories):\n"
        f"{evidence if evidence else '(no relevant evidence retrieved)'}\n\n"
        f"---\nDRAFT TO JUDGE:\n{draft}"
    )
    body = {
        "model": MODEL,
        "messages": [
            {"role": "system", "content": SYSTEM_PROMPT},
            {"role": "user", "content": user_msg},
        ],
        "max_tokens": 2500,
        "temperature": 0.0,
        "top_p": 1.0,
        "top_k": 1,
        "seed": 42,
        "stream": False,
        "chat_template_kwargs": {"enable_thinking": False},
        "stop": [
            "\n\nWait,", "\n\nActually,", "\n\nLet me", "\n\nHmm,",
            "\n\nOn second thought", "\n\nOh wait",
        ],
    }
    resp = post_json(SANHEDRIN_ENDPOINT, body, SANHEDRIN_TIMEOUT)
    if not isinstance(resp, dict):
        return ""
    try:
        msg = resp["choices"][0]["message"]
        raw = msg.get("content") or ""
        if not raw.strip():
            raw = msg.get("reasoning") or ""
    except (KeyError, IndexError, TypeError):
        return ""
    cleaned = THINK_RE.sub("", raw).strip()
    lines = [ln.strip() for ln in cleaned.splitlines() if ln.strip()]
    if not lines:
        return ""
    last = lines[-1]
    low = last.lower()
    if low.startswith("yes") or low.startswith("no"):
        return validate_verdict(last)
    for ln in reversed(lines):
        l = ln.lower()
        if l.startswith("yes") or l.startswith("no"):
            return validate_verdict(ln)
    return ""


def main() -> None:
    draft = sys.stdin.read().strip()
    if not draft:
        print("yes")
        return

    evidence, high_trust_count = fetch_evidence(draft)

    # Auto-pass if no high-trust evidence — model can't legitimately veto
    # without something concrete to cite. Eliminates the common false-positive
    # mode where the model invents a contradiction from low-trust noise.
    if high_trust_count == 0:
        print("yes")
        return

    verdict = judge(draft, evidence)

    if not verdict:
        # Fail-open: server unreachable, malformed response, etc.
        print("yes")
        return

    print(verdict)


if __name__ == "__main__":
    main()
