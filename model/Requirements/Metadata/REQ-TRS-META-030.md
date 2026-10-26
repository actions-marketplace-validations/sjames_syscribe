---
type: Requirement
id: REQ-TRS-META-030
name: "A MetadataDef can specialise another"
status: draft
reqDomain: software
reqClass: system
tags:
  - metadata
---

`supertype:` on a `MetadataDef` shall give specialisation semantics (GH #377).

## Behavior

- An application of a sub-definition counts as an application of every ancestor: `list --metadata Safety`, `has(Safety)` and checks declared on `Safety` all see an `ASIL_D` application.
- Tagged values are inherited; a sub-definition may add features and narrow ranges and enumerations, never widen them (`E329`).
- `annotates:` of a sub-definition is a subset of its parent's.
- `isAbstract: true` definitions cannot be applied (`E328`). A supertype that does not resolve or is not a `MetadataDef`, or a cycle, is `E329`.
