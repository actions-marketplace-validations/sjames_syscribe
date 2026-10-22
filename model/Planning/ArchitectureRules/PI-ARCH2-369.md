---
type: PlanningItem
id: PI-ARCH2-369
name: "Architecture rules, slice two: allow-only, layering, max-depth, waivers (GH #369)"
status: done
itemType: feature
achieves:
  - REQ-TRS-ARCH-006
  - REQ-TRS-ARCH-007
  - REQ-TRS-ARCH-008
evidence:
  - ref: TC-TRS-ARCH-006
  - ref: TC-TRS-ARCH-007
  - ref: TC-TRS-ARCH-008
  - path: repo:crates/syscribe-model/tests/arch_rules_2.rs
tags:
  - architecture-rules
---

`allow-only`, `layering` (with `layer:` on parts and as a selector key) and `max-depth` kinds, and per-rule `waivers:` with a rationale and an expiry; `E663`, `W663`, `W664`. Still open in #369: `performs:` and operation calls as dependency sources, which wait on the call-site syntax decision for operations.
