#!/usr/bin/env bash
# Generate skills/SKILL.md from `ironprose --help` output.
# Run after building: cargo build && ./scripts/generate-skills.sh
set -euo pipefail

BINARY="${BINARY:-./target/debug/ironprose}"

if [[ ! -x "$BINARY" ]]; then
  echo "Binary not found at $BINARY — run 'cargo build' first." >&2
  exit 1
fi

HELP_ROOT=$("$BINARY" --help 2>&1)

mkdir -p skills/ironprose
cat > skills/ironprose/SKILL.md << 'HEADER'
---
name: ironprose
description: Fiction prose analysis — catch weak verbs, repetition, clichés, passive voice, and other craft issues in manuscripts
metadata:
  {
    "openclaw":
      {
        "homepage": "https://github.com/benchcutlogic/ironprose-cli",
        "requires": { "bins": ["ironprose"] },
      },
  }
---

# IronProse CLI — Fiction Writing Assistant

IronProse analyzes fiction prose for craft-level issues that weaken storytelling:
repetition, weak verbs, passive voice, clichés, adverb overuse, show-don't-tell
violations, and 100+ other rules tuned for creative writing.

## Installation

```bash
# Install via npm (recommended)
npx ironprose --help

# Or install via cargo
cargo install ironprose-cli
```

## Common Workflows

### Analyze prose (always use JSON output)

```bash
# Full analysis — JSON is the only stable output format
ironprose analyze --file chapter-07.md --output json

# Pipe from stdin
cat draft.md | ironprose analyze --output json

# Raw JSON passthrough — zero translation loss, full API control
ironprose analyze \
  --json '{"text":"The dark night was very dark."}' \
  --output json

# Scores only — minimizes output tokens
ironprose analyze --file chapter-07.md --output json --score-only
```

### Schema introspection — discover before you call

```bash
ironprose schema analyze   # see request/response shapes
ironprose schema rate      # see rating payload schema
ironprose schema           # dump full OpenAPI spec
```

### Compare drafts

```bash
# Did the rewrite improve the prose?
ironprose compare \
  --original-file draft_v1.md \
  --revised-file draft_v2.md \
  --output json
```

### Pipe from editor / stdin

```bash
pbpaste | ironprose analyze --output json
```

### Rate diagnostics you disagree with

After analyzing text, rate any diagnostics that seem off. This directly
improves the engine's calibration.

```bash
# JSON passthrough — preferred for agents
ironprose rate \
  --json '{"rule":"repetition","rating":"false_positive","diagnostic_id":"d-001","context":"Intentional repetition"}'

# Convenience flags — for humans or simple ratings
ironprose rate \
  --rule repetition \
  --rating false_positive \
  --diagnostic-id d-001 \
  --context "Intentional repetition for emphasis"

# Introspect the rate schema first
ironprose schema rate
```

**Rules for rating:**

- Always include `--diagnostic-id` when available (from the analyze response)
- Use `false_positive` when the flagged issue isn't actually present
- Use `not_helpful` when the issue exists but isn't worth flagging
- Use `helpful` to confirm good diagnostics

## CLI Reference

Agents should use `ironprose <command> --help` to introspect exact arguments and flags for a specific command instead of relying purely on this document.
HEADER

# Append the help output sections
{
  echo ''
  echo '### `ironprose`'
  echo ''
  echo '```'
  echo "$HELP_ROOT"
  echo '```'
  echo ''
} >> skills/ironprose/SKILL.md

# Append static sections
cat >> skills/ironprose/SKILL.md << 'FOOTER'
## Environment Variables

| Variable            | Description                      | Default                     |
| ------------------- | -------------------------------- | --------------------------- |
| `IRONPROSE_API_URL` | API base URL                     | `https://prose-mcp.fly.dev` |
| `IRONPROSE_API_KEY` | API key for authenticated access | free tier (5000 words)      |
FOOTER

echo "✅ Generated skills/ironprose/SKILL.md"
