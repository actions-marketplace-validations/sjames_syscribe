---
type: Requirement
id: REQ-TRS-OPCALL-002
name: "A call matches the operation's signature"
status: draft
reqDomain: software
reqClass: system
tags:
  - operations
---

The arguments of a call that names an operation shall match the operation's parameters (GH #364 item 3).

## Behavior

- The arguments are the sub-action's \`parameters:\` entries (\`name\`, \`value\`), as for any call.
- An argument whose name is not a parameter of the operation, or names its return, is \`E674\`.
- An \`in\` or \`inout\` parameter of the operation with no argument is \`E674\`.
- Types and units of the values are not checked in this slice.
