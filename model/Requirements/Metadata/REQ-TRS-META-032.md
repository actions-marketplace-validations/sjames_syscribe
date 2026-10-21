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

- The library is the package `StdMetadata`, built into the tool. It is part of the model when an element refers to `StdMetadata::…`, or when `[metadata] stdlib = true`; otherwise an unused library adds nothing.
- Definitions: `Safety` (`level` 0..4, `goal`), `Optional` (`feature`), `Regulatory` (`status` verified|unverified|waived, `source`, `reviewer`), `External`, `Environment`, `Deployment` (`kind` firmware|process|container|library), `Budget` (`cpu`, `ram`).
- They are visible in `types` and `list`, typed per REQ-TRS-META-010, and can be specialised by a project.
- The library is read-only; editing or deleting one through a write route or `set` is refused.
