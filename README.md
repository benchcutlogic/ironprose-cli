# ironprose

CLI for [IronProse](https://ironprose.com) — prose analysis tools for writers.

## Install

```bash
# via npm (recommended)
npx ironprose --help

# via cargo
cargo install ironprose-cli
```

## Usage

### MCP Server (Claude Desktop, VS Code, Cursor)

Start the MCP stdio server for editor integration:

```bash
ironprose mcp --workspace /path/to/your/manuscript
```

#### Claude Desktop

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

#### VS Code (Copilot MCP)

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

### Environment Variables

| Variable              | Description                      | Default                     |
| --------------------- | -------------------------------- | --------------------------- |
| `IRONPROSE_API_URL`   | API base URL                     | `https://api.ironprose.com` |
| `IRONPROSE_API_KEY`   | API key for authenticated access | (free tier)                 |
| `IRONPROSE_WORKSPACE` | Workspace directory              | (none)                      |

## License

MIT
