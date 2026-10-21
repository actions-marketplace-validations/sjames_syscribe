---
type: Requirement
id: REQ-TRS-META-033
name: "A profile may apply stereotypes automatically"
status: draft
reqDomain: software
reqClass: system
tags:
  - metadata
---

A profile shall be able to declare rules that apply a stereotype to matching elements (GH #377).

## Behavior

- A rule has a selector (REQ-TRS-META-024) and a stereotype with optional values, e.g. every `PartDef` with `domain: software` and `isDeploymentPackage` gets `Deployment`.
- A derived application is visible to queries, checks and diagrams and is marked derived in `show`; it is never written to the element's file.
- An explicit application of the same definition wins over the derived one.
