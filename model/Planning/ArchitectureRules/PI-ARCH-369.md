---
type: PlanningItem
id: PI-ARCH-369
name: "Architecture rules, slice one: forbid-dependency and independence (GH #369)"
status: done
itemType: feature
achieves:
  - REQ-TRS-ARCH-001
  - REQ-TRS-ARCH-002
  - REQ-TRS-ARCH-003
  - REQ-TRS-ARCH-004
  - REQ-TRS-ARCH-005
evidence:
  - ref: TC-TRS-ARCH-001
  - ref: TC-TRS-ARCH-002
  - ref: TC-TRS-ARCH-003
  - ref: TC-TRS-ARCH-004
  - ref: TC-TRS-ARCH-005
  - path: repo:crates/syscribe-model/tests/arch_rules.rs
  - path: repo:crates/syscribe/tests/arch_rules_gti.rs
tags:
  - architecture-rules
---

`ArchitectureRule` (`AR-*`) with `forbid-dependency` and `independence`, evaluated by `validate` over connections, typedBy and allocations (`crates/syscribe-model/src/arch_rules.rs`); `E660`-`E662`, `W660`-`W662`. `AR-GTI-001` guards the grid-tie hardware trip. Not in this slice (issue #369): `allow-only`, `max-depth`, layers, waivers with rationale and expiry, `performs:` and operation calls as dependency sources.
