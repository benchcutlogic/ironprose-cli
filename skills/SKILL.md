---
description: How to install and use the IronProse CLI for prose analysis
---

# IronProse CLI Skill

## Installation

```bash
# Install via npm (recommended)
npx ironprose --help

# Or install via cargo
cargo install ironprose-cli
```

## Direct CLI Usage

### Analyze text

```bash
# From argument
ironprose analyze "The dark night was very dark."

# From file
ironprose analyze --file chapter1.md

# From stdin (pipe)
cat chapter1.md | ironprose analyze

# Score only (no diagnostics)
ironprose analyze --file chapter1.md --score-only

# Filter by rules
ironprose analyze --file chapter1.md --rules repetition,passive_voice

# Human-readable output
ironprose analyze --file chapter1.md --output text
```

### Compare original vs revised

```bash
ironprose compare \
  --original-file draft_v1.md \
  --revised-file draft_v2.md

# Or inline
ironprose compare \
  --original "She was very sad." \
  --revised "Grief settled into her bones."
```

### List available rules

```bash
ironprose list-rules
```

## MCP Server Usage (for AI editors)

Start the stdio MCP server for integration with Claude Desktop, VS Code Copilot, Cursor, etc:

```bash
ironprose mcp --workspace /path/to/manuscript
```

### Claude Desktop Configuration

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

### VS Code MCP Configuration

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

### Available MCP Tools

| Tool         | Description                                        |
| ------------ | -------------------------------------------------- |
| `analyze`    | Analyze prose for style, grammar, and craft issues |
| `compare`    | Compare original vs revised text                   |
| `list_rules` | List all 108 analysis rules                        |
| `rate`       | Submit feedback on diagnostics                     |
| `read_file`  | Read a file from the workspace                     |
| `write_file` | Write a file to the workspace                      |
| `list_files` | List workspace files                               |

## Environment Variables

| Variable              | Description                      | Default                     |
| --------------------- | -------------------------------- | --------------------------- |
| `IRONPROSE_API_URL`   | API base URL                     | `https://prose-mcp.fly.dev` |
| `IRONPROSE_API_KEY`   | API key for authenticated access | free tier (5000 words)      |
| `IRONPROSE_WORKSPACE` | Default workspace directory      | none                        |
