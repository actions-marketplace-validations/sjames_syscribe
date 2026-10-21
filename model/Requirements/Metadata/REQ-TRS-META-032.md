---
type: Requirement
id: REQ-TRS-META-032
name: "A standard stereotype library ships with the tool"
status: draft
reqDomain: software
reqClass: system
tags:
  - metadata
---

A read-only standard library of common stereotypes shall resolve without `E317` (GH #377).

## Behavior

- Definitions: `Safety` (`level`, `goal`), `Optional` (`feature`), `Regulatory` (`status`, `source`, `reviewer`), `External`, `Environment`, `Deployment` (`kind`), `Budget` (`cpu`, `ram`).
- They are visible in `types` and `list`, typed per REQ-TRS-META-010, and can be specialised by a project.
- The library is read-only; editing or deleting one through a write route is refused.
