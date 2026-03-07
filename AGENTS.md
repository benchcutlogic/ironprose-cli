# AGENTS.md — IronProse CLI

> This CLI is frequently invoked by AI/LLM agents.
> Always assume inputs can be adversarial.

## Project Overview

**ironprose-cli** — standalone CLI for [IronProse](https://ironprose.com) prose analysis. Proxies all analysis to the remote API (no embedded engine).

| Command                | Description                                 |
| ---------------------- | ------------------------------------------- |
| `ironprose analyze`    | Analyze prose (stdin, file, arg, or --json) |
| `ironprose compare`    | Compare original vs revised text            |
| `ironprose rate`       | Rate a diagnostic (feedback to engine)      |
| `ironprose list-rules` | List all analysis rules                     |
| `ironprose schema`     | Dump API schema for agent introspection     |

## Agent-First Usage

```bash
# Always use --output json for machine-readable output
ironprose analyze "The dark night was very dark." --output json

# Raw JSON passthrough — send the exact API payload
ironprose analyze --json '{"text": "...", "config": {"rules": ["repetition"]}}'

# Introspect API schema before constructing requests
ironprose schema analyze

# Minimize output tokens with --score-only
ironprose analyze --file chapter.md --score-only --output json

# Pipe large text via stdin
cat chapter.md | ironprose analyze --output json

# Compare drafts
ironprose compare --original "First draft." --revised "Second draft." --output json
```

### Rating Diagnostics

```bash
# Rate a diagnostic — false_positive and not_helpful ratings directly improve the engine
ironprose rate --rule repetition --rating false_positive \
  --diagnostic-id d-001 --context "Intentional repetition for emphasis"

# Agent-first: raw JSON passthrough for full API control
ironprose rate --json '{"rule":"repetition","rating":"not_helpful","diagnostic_id":"d-001"}'

# Introspect the rate schema before constructing requests
ironprose schema rate
```

## Rules for Agents

1. **Always use `--output json`** — never parse text output.
2. **Use `--score-only`** to minimize output tokens when you only need scores.
3. **Use `ironprose schema <endpoint>`** to introspect API schemas at runtime.
4. **Use `--json`** for raw payload passthrough when you need full API control.
5. **Pipe large text via stdin** — `cat chapter.md | ironprose analyze --output json`
6. **Rate diagnostics you disagree with** — `false_positive` and `not_helpful` ratings directly improve the engine.
7. **Never construct file paths with `..`** — traversal is rejected.
8. **Never use absolute file paths** — only relative paths are allowed.
9. **Do not pre-URL-encode** inputs — they will be double-encoded.

## Build & Test

```bash
cargo build                           # Debug build
cargo test                            # All tests
cargo clippy -- -D warnings           # Lint (zero warnings policy)
cargo fmt --all -- --check            # Format check
```

## Architecture

```
src/
├── main.rs     # clap CLI entrypoint (global --api-url, --api-key)
├── client.rs   # HTTP client for remote IronProse API
├── error.rs    # ApiError enum + HTTP status mapping
├── input.rs    # Input hardening (path validation, control char rejection)
├── schema.rs   # OpenAPI schema introspection (embedded spec)
└── types.rs    # Typed API request/response structs
```

### Key Design Decisions

- **No embedded engine**: All analysis runs on the remote API. Binary stays lean (~5MB).
- **Input hardening**: `input.rs` rejects path traversal, absolute paths, control chars, percent-encoding.
- **Schema introspection**: Agents self-serve API docs via `ironprose schema <endpoint>`.
- **Raw JSON passthrough**: `--json` bypasses flag-to-JSON construction for full API control.
- **Global config**: `--api-url` and `--api-key` are global clap args with env var fallbacks.

## Configuration

| Env Var             | CLI Flag    | Default                     | Description                     |
| ------------------- | ----------- | --------------------------- | ------------------------------- |
| `IRONPROSE_API_URL` | `--api-url` | `https://api.ironprose.com` | API base URL                    |
| `IRONPROSE_API_KEY` | `--api-key` | (none)                      | API key (free tier: 5000 words) |

## Error Handling

All errors are written to stderr. Parse stderr for error details:

- HTTP 401/403 → authentication failed, check `IRONPROSE_API_KEY`
- HTTP 402 → subscription required
- HTTP 429 → rate limited, wait before retrying
- HTTP 5xx → transient server error, retry after brief delay

## Release Process

Uses [changesets](https://github.com/changesets/changesets) + [cargo-dist](https://opensource.axo.dev/cargo-dist/):

```bash
pnpm changeset                        # Create changeset
# Merge PR → changeset bot creates version PR
# Merge version PR → tag push → cargo-dist builds + npm publish
```

## Rules

- Always run `cargo test` and `cargo clippy -- -D warnings` before committing
- Use conventional commit messages (`feat:`, `fix:`, `chore:`, etc.)
- Use `pnpm` (not `npm`)
- Zero clippy warnings policy
