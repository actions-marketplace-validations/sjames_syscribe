---
type: Requirement
id: REQ-TRS-ARCH-003
name: "Dependencies are derived from the architecture"
status: draft
reqDomain: software
reqClass: system
tags:
  - architecture-rules
---

The dependency graph a rule is evaluated over shall be derived from the model, not declared (GH #369).

## Behavior

- Components are the nodes. An edge `X → Y` means `X` depends on `Y`.
- `connections`: a `connections:` entry `from: a.p, to: b.q` in a component makes the definition of usage `b` depend on that of usage `a` (the consumer depends on the producer); an endpoint naming a boundary port of the owner is the owner itself.
- `typedBy`: a component depends on the definition of each usage in its `features:` (it contains it), and a `Part` on its `typedBy` definition.
- `allocations`: for an allocation (`allocatedTo:` on the source, `allocatedFrom:` on the target, or an `Allocation` element), the allocated source depends on its target (software on the host it runs on).
- `via` restricts which of the three contribute edges.
