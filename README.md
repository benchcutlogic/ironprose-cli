# ironprose

[![npm](https://img.shields.io/npm/v/ironprose)](https://www.npmjs.com/package/ironprose)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

Prose analysis CLI for [IronProse](https://ironprose.com) — style, grammar, and craft diagnostics for writers.

## Install

### npm (recommended)

```bash
npx ironprose --help
```

### Shell (macOS/Linux)

```bash
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/benchcutlogic/ironprose-cli/releases/latest/download/ironprose-cli-installer.sh | sh
```

## Usage

### Analyze prose

```bash
# From argument
ironprose analyze "The door slammed shut. She felt scared."

# From file
ironprose analyze --file chapter-01.md

# From stdin
cat draft.md | ironprose analyze

# Text output (instead of JSON)
ironprose analyze --file draft.md --output text

# Filter by rule or severity
ironprose analyze --file draft.md --rules passive_voice,repetition
ironprose analyze --file draft.md --severity-min warning
```

### Compare revisions

```bash
ironprose compare --original-file v1.md --revised-file v2.md
```

### List analysis rules

```bash
ironprose list-rules
```

### Inspect API schema

```bash
ironprose schema analyze    # schema for the analyze endpoint
ironprose schema            # full OpenAPI spec
```

## Configuration

| Variable            | Description                      | Default                     |
| ------------------- | -------------------------------- | --------------------------- |
| `IRONPROSE_API_URL` | API base URL                     | `https://api.ironprose.com` |
| `IRONPROSE_API_KEY` | API key for authenticated access | _(free tier)_               |

## License

[MIT](LICENSE)
