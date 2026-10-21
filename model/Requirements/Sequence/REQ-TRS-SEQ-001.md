---
type: Requirement
id: REQ-TRS-SEQ-001
name: "A part is a sequence subject through the actions it performs"
status: draft
reqDomain: software
reqClass: system
tags:
  - diagram
  - sequence
---

A derived `Sequence` diagram shall accept a `PartDef` or `Part` as its subject and draw the behaviour the part performs (GH #365, item 1).

## Behavior

- The subject lifeline is the part. Its messages are those of the actions named by its `performs:` entries (a string or a map with `typedBy:`), in the order the entries are declared, each action's own steps in the order `REQ-TRS-VIS-021` gives.
- Participants are resolved relative to the part, so a `via:` chain naming one of the part's own ports is a message from the part to itself.
- A `Part` usage with no `performs:` of its own uses its `typedBy` definition's.
- A part with no `performs:` entry that resolves to an `ActionDef` is `W418`, now saying to add `performs:` or to use an action as the subject; the diagram is empty.
- A diagram whose subject is an `ActionDef`, `Action`, `UseCaseDef` or `UseCase` is unchanged, byte for byte.
