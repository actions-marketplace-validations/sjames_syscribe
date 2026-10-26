---
type: PlanningItem
id: PI-META-379
name: "Stereotype-conditioned checks, gates and reports (GH #379)"
status: done
itemType: feature
achieves:
  - REQ-TRS-META-050
  - REQ-TRS-META-051
  - REQ-TRS-META-052
  - REQ-TRS-META-053
  - REQ-TRS-META-054
evidence:
  - ref: TC-TRS-META-050
  - ref: TC-TRS-META-051
  - ref: TC-TRS-META-052
  - ref: TC-TRS-META-053
  - ref: TC-TRS-META-054
  - path: repo:crates/syscribe/tests/metadata_checks.rs
  - path: repo:crates/syscribe/tests/coverage_gate.rs
tags:
  - metadata
---

`checks:` with `E650`/`W650` (050), `validate --deny-meta` and `[gates] deny_meta` (051), the `meta` report (052), selectors in a TestPlan, `matrix`, a view's `include:`, `[[coverage.rule]]` and an `ArchitectureRule`, plus the per-stereotype coverage threshold `[[gates.coverage]]` (053), and the baseline refusal (054).
