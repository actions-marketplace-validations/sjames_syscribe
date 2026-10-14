---
type: Requirement
id: REQ-TRS-RESPLUG-001
name: "A results plugin converts any test report into normalised result records"
status: draft
reqDomain: software
reqClass: system
tags:
  - results
  - plugins
---

A project shall be able to ingest a test report of any format by configuring an external converter, with no change to Syscribe: `ingest-results --format plugin:<alias> <file>`. This extends the stdio-plugin mechanism (ADR-SYS-PLUGIN-002) from model elements to results; hardware-in-the-loop tools, rig scripts and proprietary logs are the intended users.

## Behavior

- **Configuration.** `[plugins.<alias>]` in `.syscribe.toml` with the existing keys (`command`, `args`, `timeout_ms`) and a new `kind = "results"`. A plugin without `kind` stays a foreign-format plugin, so existing configurations are unchanged. A `foreignFormat:` package naming a results plugin, or `--format plugin:` naming a foreign-format one, is an error.
- **Protocol.** Syscribe spawns the command, writes one JSON request to its stdin (`protocolVersion`, `alias`, `reportPath` and `modelRoot`, both absolute, plus `run`, `config` and `environment` when given on the command line) and closes it. The plugin writes one JSON object to stdout: `{records, diagnostics}`, where `records` are the normalised records of REQ-TRS-RESREC-001 and `diagnostics` are messages relayed to the user.
- **Same validator.** Plugin output goes through exactly the record validator of REQ-TRS-RESREC-001; a plugin cannot store anything the built-in format could not. A malformed record rejects the whole ingest, nothing is written.
- **Failure handling.** A command that cannot be spawned, exits non-zero, exceeds `timeout_ms` or writes an invalid envelope fails the ingest with a finding naming the alias and the cause (new codes, allocated from the free range at implementation). No sandbox: a plugin is a plain OS subprocess with the user's rights, the same trust level as the other plugins, stated in the documentation.
- **Command-line values win.** `--environment` and `--build-commit` given to `ingest-results` fill provenance a record leaves empty; a value in the record is kept.
- **Inspection.** `plugins run <alias> --report <file> --dry-run` invokes a results plugin and prints its raw envelope, with no merge and no validation, like the existing dry run.
- **Determinism.** The ingest is a pure function of the report, the plugin output and the command-line values, apart from the ingest timestamp.

## Documentation and spec (part of the acceptance of this requirement)

`docs/model-guide/stdio-plugins.md` (a results section with the protocol and trust note); the `plugins` and `ingest-results` help pages; `docs/cli/index.md`; an addendum to ADR-SYS-PLUGIN-002 recording the second protocol; the spec chapter on plugins and both validation catalogues for the new codes; a worked example `examples/stdio-plugins/results-python/` (a converter for a made-up rig log) and, as a further example, a converter for TAP (Robot and Cucumber JSON are built in, REQ-TRS-RESROBOT-001 and REQ-TRS-RESCUKE-001); and the drift tests.
