---
name: skills
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

### `ironprose`

```
IronProse CLI — prose analysis tools for writers

Usage: ironprose [OPTIONS] <COMMAND>

Commands:
  analyze     Analyze prose text for style, grammar, and craft issues
  compare     Compare original and revised text
  list-rules  List all available analysis rules
  schema      Dump the API schema for an endpoint (agent introspection)
  help        Print this message or the help of the given subcommand(s)

Options:
      --api-url <API_URL>  IronProse API base URL [env: IRONPROSE_API_URL=] [default: https://api.ironprose.com]
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
      --api-url <API_URL>            IronProse API base URL [env: IRONPROSE_API_URL=] [default: https://api.ironprose.com]
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
      --api-url <API_URL>              IronProse API base URL [env: IRONPROSE_API_URL=] [default: https://api.ironprose.com]
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
      --api-url <API_URL>  IronProse API base URL [env: IRONPROSE_API_URL=] [default: https://api.ironprose.com]
      --api-key <API_KEY>  API key for authenticated access (optional, free tier available) [env: IRONPROSE_API_KEY=]
  -h, --help               Print help
```

## Environment Variables

| Variable            | Description                      | Default                     |
| ------------------- | -------------------------------- | --------------------------- |
| `IRONPROSE_API_URL` | API base URL                     | `https://api.ironprose.com` |
| `IRONPROSE_API_KEY` | API key for authenticated access | free tier (5000 words)      |
