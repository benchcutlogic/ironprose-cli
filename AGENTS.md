# AGENTS.md — ironprose-cli

> **Reference for AI agents** working in this repo.

## Project Overview

**ironprose-cli** — standalone CLI for [IronProse](https://ironprose.com) prose analysis. Proxies all analysis to the remote API (no embedded engine).

| Command                | Description                                          |
| ---------------------- | ---------------------------------------------------- |
| `ironprose mcp`        | Stdio MCP server for Claude Desktop, VS Code, Cursor |
| `ironprose analyze`    | Analyze prose (stdin, file, or arg)                  |
| `ironprose compare`    | Compare original vs revised text                     |
| `ironprose list-rules` | List all analysis rules                              |

## Build & Test

```bash
cargo build                           # Debug build
cargo test                            # All tests (28 unit tests)
cargo clippy -- -D warnings           # Lint (zero warnings policy)
cargo fmt --all -- --check            # Format check
```

## Architecture

```
src/
├── main.rs              # clap CLI entrypoint (global --api-url, --api-key)
└── mcp/
    ├── mod.rs            # MCP module entry
    ├── server.rs         # rmcp MCP server (tool routing, schemas)
    ├── proxy.rs          # HTTP proxy to remote IronProse API
    ├── local_tools.rs    # read_file, write_file, list_files
    ├── sandbox.rs        # Path validation (workspace containment)
    ├── audit.rs          # Write operation audit log
    └── error.rs          # MCP error mapping (HTTP → MCP errors)
```

### Key Design Decisions

- **No embedded engine**: All analysis runs on the remote API. Binary stays lean (~5MB).
- **Sandboxed file access**: `sandbox.rs` enforces workspace containment. No absolute paths, no traversal.
- **Global config**: `--api-url` and `--api-key` are global clap args with env var fallbacks (`IRONPROSE_API_URL`, `IRONPROSE_API_KEY`).
- **MCP server**: Uses `rmcp` 0.15 with `schemars` v1. Stdio transport only.

## Release Process

Uses [changesets](https://github.com/changesets/changesets) + [cargo-dist](https://opensource.axo.dev/cargo-dist/):

```bash
pnpm changeset                        # Create changeset
# Merge PR → changeset bot creates version PR
# Merge version PR → tag push → cargo-dist builds + npm publish
```

### Distribution

| Method               | Command                                   |
| -------------------- | ----------------------------------------- |
| npm                  | `npx ironprose --help`                    |
| Cargo                | `cargo install ironprose-cli`             |
| Shell (macOS/Linux)  | `curl --proto '=https' -LsSf <url> \| sh` |
| PowerShell (Windows) | `irm <url> \| iex`                        |

## Rules

- Always run `cargo test` and `cargo clippy -- -D warnings` before committing
- Use conventional commit messages (`feat:`, `fix:`, `chore:`, etc.)
- Use `pnpm` (not `npm`)
- Zero clippy warnings policy
- Test fixtures in `tests/fixtures/` (Frankenstein, Sherlock Holmes — public domain)

## Configuration

| Env Var               | CLI Flag      | Default                     | Description                      |
| --------------------- | ------------- | --------------------------- | -------------------------------- |
| `IRONPROSE_API_URL`   | `--api-url`   | `https://api.ironprose.com` | API base URL                     |
| `IRONPROSE_API_KEY`   | `--api-key`   | (none)                      | API key (free tier: 5000 words)  |
| `IRONPROSE_WORKSPACE` | `--workspace` | (none)                      | Workspace dir for MCP file tools |

## Files Reference

| File                      | Purpose                                      |
| ------------------------- | -------------------------------------------- |
| `dist-workspace.toml`     | cargo-dist config (targets, installers, npm) |
| `.changeset/config.json`  | Changesets config                            |
| `scripts/sync-version.sh` | Syncs changeset version → Cargo.toml         |
| `scripts/tag-release.sh`  | Creates git tag for cargo-dist release       |
| `skills/SKILL.md`         | Agent discoverability (usage examples)       |
| `tests/fixtures/`         | Public domain novels for testing             |
