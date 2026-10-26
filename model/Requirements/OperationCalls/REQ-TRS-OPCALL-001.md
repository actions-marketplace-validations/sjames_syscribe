---
type: Requirement
id: REQ-TRS-OPCALL-001
name: "A call names the operation it calls"
status: draft
reqDomain: software
reqClass: system
tags:
  - operations
---

A \`SendAction\` or \`PerformAction\` sub-action shall be able to name the operation it calls with \`operation:\` (GH #364 item 3, #398).

## Behavior

- \`operation:\` is \`<owner>::<operation name>\`, the owner being any element that declares \`operations:\` (a \`PortDef\`, \`InterfaceDef\`, \`ConnectionDef\` or \`PartDef\`). It is found by resolving the whole reference, else the reference without its last segment as the owner.
- An owner that does not resolve, or that declares no operation of that name, is \`E673\`, naming the call.
- \`operation:\` on any other kind of sub-action is \`E673\`.
- A sub-action with no \`operation:\` is not looked at.
