---
type: Requirement
id: REQ-TRS-UNIT-002
name: "A software unit is traced to code, requirement, test and component"
status: draft
reqDomain: software
reqClass: system
tags:
  - unit-level
---

A part with \`level: unit\` and a status other than \`draft\` shall be traced downward to code and tests and upward to a requirement and a component.

## Behavior

- No \`implementedBy:\`: \`E741\`. An \`implementedBy:\` entry that is a local path naming no symbol (\`path#symbol\`): \`W740\`.
- Satisfies no requirement: \`E742\`.
- No unit-level test, an L1 \`TestCase\` that verifies the unit or a requirement it satisfies: \`E743\`.
- Contained by no part of \`level: component\` (through \`features:\`): \`E744\`.
