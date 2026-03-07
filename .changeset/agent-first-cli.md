---
---

### Agent-First CLI Optimization

- **Schema introspection**: `ironprose schema [endpoint]` dumps API schemas at runtime (remote-first with local cache, embedded fallback)
- **Raw JSON passthrough**: `--json` flag for `analyze` and `compare` sends payloads directly to the API
- **Input hardening**: Rejects path traversal, absolute paths, control characters, and percent-encoding
- **AGENTS.md**: Agent-specific guidance for AI consumers
- **CI smoketest**: Builds the release binary and exercises key commands
- **Cross-platform fix**: Absolute path rejection now works on Windows
