# ironprose

[![npm](https://img.shields.io/npm/v/ironprose)](https://www.npmjs.com/package/ironprose)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

**Grammarly catches commas. AI tries to write the book for you.**
**IronProse actually helps you edit.**

103+ deterministic craft analyzers — no AI hallucinations, no guessing. Every diagnostic is computed from your actual prose.

```bash
npx ironprose analyze "The dark night was very dark and she felt scared."
```

```json
{
  "score": 42,
  "diagnostics": [
    {
      "rule": "repetition",
      "severity": "warning",
      "message": "\"dark\" repeated 2× within 8 words"
    },
    {
      "rule": "filter_words",
      "severity": "info",
      "message": "\"felt\" distances the reader from the emotion"
    },
    {
      "rule": "adverb_density",
      "severity": "info",
      "message": "\"very\" weakens the adjective it modifies"
    }
  ]
}
```

## What it catches

| Category                    | Examples                                                         |
| --------------------------- | ---------------------------------------------------------------- |
| **Grammar & Mechanics**     | Comma splices · dangling modifiers · tense consistency           |
| **Word Choice & Economy**   | Passive voice · filter words · adverb density · hedging          |
| **Craft & Style**           | Show don't tell · purple prose · clichés · emotional restraint   |
| **Sentence & Rhythm**       | Repetitive structure · echoed openers · monotonous length        |
| **Readability & Structure** | Pacing variance · paragraph balance · readability score          |
| **Dialogue**                | Said-ism soup · "As you know, Bob" · dialogue-to-narration ratio |
| **Character**               | Voice fingerprinting · white room syndrome · emotional residue   |

[Browse all 103+ craft lenses →](https://ironprose.com#lenses)

## Install

```bash
# npm (recommended)
npx ironprose --help

# macOS/Linux
curl --proto '=https' --tlsv1.2 -LsSf \
  https://github.com/benchcutlogic/ironprose-cli/releases/latest/download/ironprose-cli-installer.sh | sh

# Agent skills (for AI coding agents)
npx skills install benchcutlogic/ironprose-cli
```

## Usage

```bash
# Analyze text (JSON output — agents should always use this)
ironprose analyze --file chapter-01.md --output json
cat draft.md | ironprose analyze --output json

# Scores only — minimizes output tokens
ironprose analyze --file draft.md --output json --score-only

# Raw JSON passthrough — zero translation loss
ironprose analyze \
  --json '{"text":"The dark night was very dark."}' \
  --output json

# Schema introspection — discover endpoints at runtime
ironprose schema analyze
ironprose schema rate

# Rate diagnostics — closes the feedback loop
ironprose rate \
  --json '{"rule":"repetition","rating":"false_positive","diagnostic_id":"d-001"}'

# Compare revisions
ironprose compare --original-file v1.md --revised-file v2.md --output json

# List all rules
ironprose list-rules --output json

# Human-readable output
ironprose analyze --file draft.md --output text
ironprose rate --rule repetition --rating helpful --diagnostic-id d-001
```

### `--output text` stream split

When `--output text` is used, the two output streams are **intentionally separate**:

| Stream   | Content                               |
| -------- | ------------------------------------- |
| `stderr` | Diagnostics (one per line)            |
| `stdout` | Score JSON                            |

To capture **both** in a single file or variable, merge stderr into stdout:

```bash
ironprose analyze --file draft.md --output text 2>&1
```

To capture them separately:

```bash
ironprose analyze --file draft.md --output text \
  > score.json \
  2> diagnostics.txt
```

### Line numbers

Diagnostics include `start_line` / `end_line` fields that come from the API in **0-indexed** form (line 1 of the file = `0`). The `--output text` renderer converts these to **1-indexed** line numbers for human display (`L1`, `L2`, …). Raw `--output json` output preserves the 0-indexed values as returned by the API.

## Configuration

| Variable            | Description                      | Default                     |
| ------------------- | -------------------------------- | --------------------------- |
| `IRONPROSE_API_URL` | API base URL                     | `https://api.ironprose.com` |
| `IRONPROSE_API_KEY` | API key for authenticated access | _(free tier)_               |

## License

[MIT](LICENSE)
