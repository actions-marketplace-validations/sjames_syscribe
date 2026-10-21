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

Stereotype selectors shall be accepted by architecture rules, views and coverage filters, with thresholds per stereotype (GH #379, #369). The selector grammar, the TestPlan `selection.stereotypes:` and `matrix --select` are done (REQ-TRS-META-024); a `stereotype` selector in `[[coverage.rule]]` (any of; specialisations count; applied by `coverage tree`, `matrix --rollup`, `W305` and the `E898` integrity guard) is done. A diagram view's `include: {stereotypes: […]}` (REQ-TRS-META-024) is done. Architecture rules (#369) are not. (There is no numeric threshold to raise: `[coverage]` chooses how a parent is judged complete, and `both` is already the strictest.)

## Behavior

- A `[[coverage.rule]]` may select requirements by `stereotype`, so a stereotype can choose how its parents are judged complete. A rule that loosens an integrity-rated parent is still `E898`.
