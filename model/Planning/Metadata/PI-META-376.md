---
type: PlanningItem
id: PI-META-376
name: "Query and select by stereotype and tagged value (GH #376)"
status: done
itemType: feature
achieves:
  - REQ-TRS-META-020
  - REQ-TRS-META-021
  - REQ-TRS-META-022
  - REQ-TRS-META-023
  - REQ-TRS-META-024
  - REQ-TRS-META-025
evidence:
  - ref: TC-TRS-META-020
  - ref: TC-TRS-META-021
  - ref: TC-TRS-META-022
  - ref: TC-TRS-META-023
  - ref: TC-TRS-META-024
  - ref: TC-TRS-META-025
  - path: repo:crates/syscribe/tests/metadata_query.rs
  - path: repo:crates/syscribe-model/tests/metadata_selector.rs
  - path: repo:crates/syscribe/tests/metadata_select_cli.rs
tags:
  - metadata
---

`--where meta.…` (020), repeatable/comma/negated `--metadata` (021), `metadata` in `list --json` (022), `has()`/`meta()` (023), the common selector `{stereotype, where}` with TestPlan `selection.stereotypes:` and `matrix --select` (024), and `appliesWhen:` on an application with `diff` (025). Diagram `include:` and architecture rules will take the selector when #334/#369 exist.
