---
type: Requirement
id: REQ-TRS-UNIT-001
name: "A software part declares its design level"
status: draft
reqDomain: software
reqClass: system
tags:
  - unit-level
---

A software \`PartDef\`/\`Part\` shall be able to declare \`level: component | unit\` (GH #371).

## Behavior

- Any other value is \`E740\`.
- \`level\` on any other element type is \`W047\`.
- Nothing else is checked for a part that declares no level, or \`level: component\`.
