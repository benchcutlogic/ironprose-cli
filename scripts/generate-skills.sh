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
HELP_ANALYZE=$("$BINARY" analyze --help 2>&1)
HELP_COMPARE=$("$BINARY" compare --help 2>&1)
HELP_LIST_RULES=$("$BINARY" list-rules --help 2>&1)
HELP_MCP=$("$BINARY" mcp --help 2>&1)

cat > skills/SKILL.md << 'HEADER'
---
name: skills
description: Fiction prose analysis — catch weak verbs, repetition, clichés, passive voice, and other craft issues in manuscripts
metadata: {"openclaw": {"homepage": "https://github.com/benchcutlogic/ironprose-cli", "requires": {"bins": ["ironprose"]}}}
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

### Revise a chapter draft

```bash
# Full analysis with human-readable output
ironprose analyze --file chapter-07.md --output text

# Focus on specific craft issues
ironprose analyze --file chapter-07.md --rules repetition,weak_verb,passive_voice

# Score only — quick health check before submitting
ironprose analyze --file chapter-07.md --score-only
```

### Compare drafts during revision

```bash
# Did the rewrite actually improve the prose?
ironprose compare --original-file draft_v1.md --revised-file draft_v2.md
```

### Pipe from editor / stdin

```bash
# Analyze selected text from clipboard
pbpaste | ironprose analyze --output text
```

## CLI Reference

HEADER

# Append the help output sections
{
  echo '### `ironprose`'
  echo ''
  echo '```'
  echo "$HELP_ROOT"
  echo '```'
  echo ''
  echo '### `ironprose analyze`'
  echo ''
  echo '```'
  echo "$HELP_ANALYZE"
  echo '```'
  echo ''
  echo '### `ironprose compare`'
  echo ''
  echo '```'
  echo "$HELP_COMPARE"
  echo '```'
  echo ''
  echo '### `ironprose list-rules`'
  echo ''
  echo '```'
  echo "$HELP_LIST_RULES"
  echo '```'
  echo ''
  echo '### `ironprose mcp`'
  echo ''
  echo '```'
  echo "$HELP_MCP"
  echo '```'
} >> skills/SKILL.md

# Append static sections
cat >> skills/SKILL.md << 'FOOTER'

## MCP Server (AI Editor Integration)

When used as an MCP server, IronProse gives your AI assistant direct access to
prose analysis tools — so it can analyze chapters, compare drafts, and suggest
revisions without copy-pasting text.

### Claude Desktop

Add to `~/Library/Application Support/Claude/claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "ironprose": {
      "command": "npx",
      "args": ["-y", "ironprose", "mcp", "--workspace", "/path/to/manuscript"]
    }
  }
}
```

### VS Code / Cursor

Add to `.vscode/mcp.json`:

```json
{
  "servers": {
    "ironprose": {
      "command": "npx",
      "args": ["-y", "ironprose", "mcp", "--workspace", "${workspaceFolder}"]
    }
  }
}
```

## Environment Variables

| Variable              | Description                      | Default                     |
| --------------------- | -------------------------------- | --------------------------- |
| `IRONPROSE_API_URL`   | API base URL                     | `https://api.ironprose.com` |
| `IRONPROSE_API_KEY`   | API key for authenticated access | free tier (5000 words)      |
| `IRONPROSE_WORKSPACE` | Default workspace directory      | none                        |
FOOTER

echo "✅ Generated skills/SKILL.md"
