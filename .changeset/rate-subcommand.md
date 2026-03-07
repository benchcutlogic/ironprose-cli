---
"ironprose-cli": minor
---

Add `rate` subcommand for agent diagnostic feedback

- New `ironprose rate` command with `--json` as primary agent path and `--rule`/`--rating` convenience flags
- Telemetry Bridge fields (`source_type`, `confidence`) surfaced on `DiagnosticItem`
- Text output shows `[Heuristic 1.00] [id:d-001]` tags and rating hint on stderr
- AGENTS.md rule #6: "Rate diagnostics you disagree with"
- SKILL.md rating workflow section with examples
- 4 new integration tests for rate command
