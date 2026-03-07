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
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/benchcutlogic/ironprose-cli/releases/latest/download/ironprose-cli-installer.sh | sh
```

## Usage

```bash
# Analyze a file
ironprose analyze --file chapter-01.md

# Pipe from stdin
cat draft.md | ironprose analyze

# Human-readable output
ironprose analyze --file draft.md --output text

# Filter by rule or severity
ironprose analyze --file draft.md --rules passive_voice,repetition
ironprose analyze --file draft.md --severity-min warning

# Compare revisions
ironprose compare --original-file v1.md --revised-file v2.md

# List all rules
ironprose list-rules

# Inspect API schema
ironprose schema analyze

# Rate a diagnostic (closes the feedback loop)
ironprose rate --rule repetition --rating false_positive --diagnostic-id d-001
```

## Configuration

| Variable            | Description                      | Default                     |
| ------------------- | -------------------------------- | --------------------------- |
| `IRONPROSE_API_URL` | API base URL                     | `https://api.ironprose.com` |
| `IRONPROSE_API_KEY` | API key for authenticated access | _(free tier)_               |

## License

[MIT](LICENSE)
