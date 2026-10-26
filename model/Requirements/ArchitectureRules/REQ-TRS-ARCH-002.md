---
type: Requirement
id: REQ-TRS-ARCH-002
name: "Rule selectors choose components"
status: draft
reqDomain: software
reqClass: system
tags:
  - architecture-rules
---

A selector shall choose the components a rule is about (GH #369).

## Behavior

- The components are the `PartDef` and `Part` elements of the model.
- A selector is a mapping whose keys are ANDed: `package` (a qualified-name prefix), `tag`, `domain` (`system`, `hardware`, `software`), `stereotype` (a stereotype selector, REQ-TRS-META-024).
- An unknown key, an empty selector or a malformed `stereotype` is `E662`.
- A selector that chooses no component is `W662` on the rule: the rule would pass without checking anything.
