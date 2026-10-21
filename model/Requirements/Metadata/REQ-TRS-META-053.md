---
type: Requirement
id: REQ-TRS-META-053
name: "Selectors and thresholds work in rules, views and coverage"
status: draft
reqDomain: software
reqClass: system
tags:
  - metadata
---

Stereotype selectors shall be accepted by architecture rules, views and coverage filters, with thresholds per stereotype (GH #379, #369). The selector grammar, the TestPlan `selection.stereotypes:` and `matrix --select` are done (REQ-TRS-META-024); architecture rules, diagram `include:` and per-stereotype coverage thresholds are not.

## Behavior

- Coverage thresholds may be set per stereotype, higher for `Safety` than the default.
