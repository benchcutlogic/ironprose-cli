# ironprose-cli

## 0.3.1

### Patch Changes

- d947aa0: docs: wrap long CLI command examples in README and SKILL.md to prevent horizontal overflow on npmjs.com

## 0.3.0

### Minor Changes

- baf06b0: Add `rate` subcommand for agent diagnostic feedback

  - New `ironprose rate` command with `--json` as primary agent path and `--rule`/`--rating` convenience flags
  - Telemetry Bridge fields (`source_type`, `confidence`) surfaced on `DiagnosticItem`
  - Text output shows `[Heuristic 1.00] [id:d-001]` tags and rating hint on stderr
  - AGENTS.md rule #6: "Rate diagnostics you disagree with"
  - SKILL.md rating workflow section with examples
  - 4 new integration tests for rate command

## 0.2.1

### Patch Changes

- 18d488e: Use JPOEHNELT_BOT PAT for workflow permissions and update README with brand-aligned copy.
- d1498bb: Clean up README with badges, CLI usage examples, and improved structure for both human and AI readers.

## 0.2.0

### Minor Changes

- 6ac9465: ### Agent-First CLI Optimization

  - **Schema introspection**: `ironprose schema [endpoint]` dumps API schemas at runtime (remote-first with local cache, embedded fallback)
  - **Raw JSON passthrough**: `--json` flag for `analyze` and `compare` sends payloads directly to the API
  - **Input hardening**: Rejects path traversal, absolute paths, control characters, and percent-encoding
  - **AGENTS.md**: Agent-specific guidance for AI consumers
  - **CI smoketest**: Builds the release binary and exercises key commands
  - **Cross-platform fix**: Absolute path rejection now works on Windows
