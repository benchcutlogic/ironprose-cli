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

### `ironprose`

```
IronProse CLI — prose analysis tools for writers

Usage: ironprose [OPTIONS] <COMMAND>

Commands:
  analyze     Analyze prose text for style, grammar, and craft issues
  compare     Compare original and revised text
  rate        Rate a diagnostic as helpful, not_helpful, or false_positive
  list-rules  List all available analysis rules
  schema      Dump the API schema for an endpoint (agent introspection)
  help        Print this message or the help of the given subcommand(s)

Options:
      --api-url <API_URL>  IronProse API base URL [env: IRONPROSE_API_URL=] [default: https://prose-mcp.fly.dev]
      --api-key <API_KEY>  API key for authenticated access (optional, free tier available) [env: IRONPROSE_API_KEY=]
  -h, --help               Print help
  -V, --version            Print version
```

### `ironprose analyze`

```
Analyze prose text for style, grammar, and craft issues

Usage: ironprose analyze [OPTIONS] [TEXT]

Arguments:
  [TEXT]  Text to analyze (reads from stdin if not provided)

Options:
  -f, --file <FILE>                  Read input from a file
      --json <JSON>                  Raw JSON payload (sent directly to the API, bypasses other flags)
      --score-only                   Only output scores (no diagnostics)
      --rules <RULES>                Only run specific rules (comma-separated)
      --api-url <API_URL>            IronProse API base URL [env: IRONPROSE_API_URL=] [default: https://prose-mcp.fly.dev]
      --severity-min <SEVERITY_MIN>  Minimum severity: error, warning, information, hint
      --api-key <API_KEY>            API key for authenticated access (optional, free tier available) [env: IRONPROSE_API_KEY=]
  -o, --output <OUTPUT>              Output format: json (default), or text [default: json]
  -h, --help                         Print help
```

### `ironprose compare`

```
Compare original and revised text

Usage: ironprose compare [OPTIONS]

Options:
      --original <ORIGINAL>            Original text (or use --original-file)
      --revised <REVISED>              Revised text (or use --revised-file)
      --original-file <ORIGINAL_FILE>  Read original from file
      --revised-file <REVISED_FILE>    Read revised from file
      --api-url <API_URL>              IronProse API base URL [env: IRONPROSE_API_URL=] [default: https://prose-mcp.fly.dev]
      --json <JSON>                    Raw JSON payload (sent directly to the API, bypasses other flags)
      --api-key <API_KEY>              API key for authenticated access (optional, free tier available) [env: IRONPROSE_API_KEY=]
  -o, --output <OUTPUT>                Output format: json (default), or text [default: json]
  -h, --help                           Print help
```

### `ironprose list-rules`

```
List all available analysis rules

Usage: ironprose list-rules [OPTIONS]

Options:
      --api-url <API_URL>  IronProse API base URL [env: IRONPROSE_API_URL=] [default: https://prose-mcp.fly.dev]
      --api-key <API_KEY>  API key for authenticated access (optional, free tier available) [env: IRONPROSE_API_KEY=]
  -h, --help               Print help
```

## Environment Variables

| Variable            | Description                      | Default                     |
| ------------------- | -------------------------------- | --------------------------- |
| `IRONPROSE_API_URL` | API base URL                     | `https://prose-mcp.fly.dev` |
| `IRONPROSE_API_KEY` | API key for authenticated access | free tier (5000 words)      |

### `ironprose rate`

```
Rate a diagnostic as helpful, not_helpful, or false_positive

Usage: ironprose rate [OPTIONS]

Options:
      --rule <RULE>                    Rule that produced the diagnostic
      --rating <RATING>                Rating: helpful, not_helpful, or false_positive
      --json <JSON>                    Raw JSON payload (sent directly to the API, bypasses other flags)
      --context <CONTEXT>              Why this rating was given (free-text context)
      --diagnostic-id <DIAGNOSTIC_ID>  Diagnostic ID from the analyze response
      --api-url <API_URL>              IronProse API base URL [env: IRONPROSE_API_URL=] [default: https://prose-mcp.fly.dev]
      --api-key <API_KEY>              API key for authenticated access [env: IRONPROSE_API_KEY=]
  -h, --help                           Print help
```
