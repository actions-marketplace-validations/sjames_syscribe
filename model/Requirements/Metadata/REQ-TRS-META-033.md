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

- `autoApply:` on a profile package lists rules `{select: {type, stereotype?, where: {field: value, …}}, apply: {type: Def, key: value, …}}`. `where` compares frontmatter fields (e.g. every `PartDef` with `domain: software` and `isDeploymentPackage: true` gets `Deployment`). The shared selector grammar of REQ-TRS-META-024 may later replace `select`.
- Only the rules of *enabled* profiles apply.
- A derived application is visible to queries, `has()`/`meta()`, `list --json` (`derived: true`) and `show` (`(derived)`); it is never written to the element's file. Diagrams and the typing/constraint checks read written applications only.
- An explicit application of the same definition wins over the derived one.
