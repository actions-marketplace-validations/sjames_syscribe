---
type: PartDef
name: SomePart
satisfies:
  - REQ-TST-001
---

PartDef satisfying a SIL 2 requirement but declaring no `silLevel:` — should produce E843 (satisfying element must match or exceed the requirement's integrity level).
