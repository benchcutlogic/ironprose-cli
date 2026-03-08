# ironprose-cli

## 2026.3.7

### Changes

- Switched to CalVer (YYYY.M.D) versioning scheme
- Extensive agent-driven testing across Gutenberg corpus (Pride & Prejudice, Great Gatsby, Dorian Gray, Call of the Wild, Dubliners) — 156 diagnostics reviewed and rated
- Filed bugs: list-rules --output flag (#23), compare duplicate diagnostics (#24), schema export inconsistency (#25), --output text stream split (#26), body_language_cliches phantom match (#50), shell quoting in --json (#51)
- Filed enhancements: --genre/--locale CLI flags (#27), insights subcommand (#28), format awareness (#52), sentence_complexity literary threshold (#56), intensifier era awareness (#55), prop_permanence narrowing (#54)

## 0.5.2

### Patch Changes

- 7eadc59: fix: compare --output text now shows fixed/introduced/persistent diagnostics and score delta (closes #47)

## 0.5.1

### Patch Changes

- 0445ee8: fix: validate --locale flag client-side, reject unknown values with exit 1 (closes #37)

## 0.5.0

### Minor Changes

- e1ac7ff: feat: add --genre and --locale flags to compare subcommand (closes #38)

### Patch Changes

- e840636: fix: resolve $ref in schema output so agents can discover all request body parameters (closes #39)

## 0.4.0

### Minor Changes

- 7377bad: Add --genre and --locale flags to analyze subcommand
- 9f038b4: feat: add insights subcommand (closes #28)

### Patch Changes

- af5822e: Added `insights` and `export` to the CLI's schema mapping so agents can introspect the endpoints.
- 0d1e061: Deduplicate diagnostics in compare introduced/fixed arrays
- d12ecf7: Add --output flag to list-rules subcommand
- ca5abbd: Document --output text stream split and fix 0-indexed line numbers
- 99fd4c0: Remove non-existent export endpoint from schema error message

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
