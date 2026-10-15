---
type: PlanningItem
id: PI-RESPLUG-001
name: "Results plugins for any report format"
status: done
itemType: feature
achieves: [REQ-TRS-RESPLUG-001]
evidence:
  - ref: TC-TRS-RESPLUG-001
  - path: repo:crates/syscribe/tests/results_plugins.rs
  - path: repo:crates/syscribe-model/src/plugins/results.rs
  - path: repo:examples/stdio-plugins/results-python/README.md
tags:
  - results
---

`PluginEntry.kind`, the results protocol (`plugins/results.rs`, `invoke_raw` made generic over the request), `ingest-results --format plugin:<alias>` and `--build-commit`, `plugins run <alias> --report … --dry-run`, `E552` (kind mismatch, also emitted through the derived-finding table) and `E553` (results-plugin failure).

Documentation done: `docs/model-guide/stdio-plugins.md` (section 10), the `plugins` and `ingest-results` help, the CLI reference, the spec, both catalogues and the index range, the ADR addendum on `ADR-SYS-PLUGIN-002`, and the worked example (a rig-log converter and a TAP converter with sample reports).

Known limits: the request carries paths only, so a plugin reads the report itself; there is no sandbox (the trust level of the foreign-format plugins); a plugin runs on every ingest and nothing is cached.
