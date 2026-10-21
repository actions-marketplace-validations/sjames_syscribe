---
type: PlanningItem
id: PI-META-375
name: "Typed tagged values (GH #375)"
status: done
itemType: feature
achieves:
  - REQ-TRS-META-010
  - REQ-TRS-META-011
  - REQ-TRS-META-012
  - REQ-TRS-META-013
  - REQ-TRS-META-014
  - REQ-TRS-META-015
evidence:
  - ref: TC-TRS-META-010
  - ref: TC-TRS-META-011
  - ref: TC-TRS-META-012
  - ref: TC-TRS-META-013
  - ref: TC-TRS-META-014
  - ref: TC-TRS-META-015
  - path: repo:crates/syscribe-model/tests/metadata_typed.rs
  - path: repo:crates/syscribe/tests/metadata_typed_cli.rs
tags:
  - metadata
---

Findings `E322`–`E327` (`crates/syscribe-model/src/tagged_values.rs`), defaults in `show` and `list --json`, tagged-value references in `refs`/`impact`/the delete guard, placeholders under `--config`. `list --where meta.…` over defaults waits for #376 (one ignored test).
