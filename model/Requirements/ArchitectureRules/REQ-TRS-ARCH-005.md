---
type: Requirement
id: REQ-TRS-ARCH-005
name: "independence reports the shared element"
status: draft
reqDomain: software
reqClass: system
tags:
  - architecture-rules
---

An `independence` rule shall report every element two sets both depend on (GH #369).

## Behavior

- The closure of a set is its components and everything they depend on, transitively. The rule holds when the closure of the `from` set and the closure of the `to` set are disjoint.
- Otherwise one finding per shared component that causes the sharing — not what it contains or depends on, which is shared as a consequence — on the rule's file, naming the rule, the shared component and a path from each set to it; `E661`/`W661` by severity.
- Wiring one set to a component of the other makes that component shared; removing the sharing clears the finding.
