---
type: Requirement
id: REQ-TRS-META-013
name: "Tagged values may reference elements"
status: draft
reqDomain: software
reqClass: system
tags:
  - metadata
---

A tagged value shall be able to be a reference to a model element, checked and traced like other references (GH #375).

## Behavior

- A feature whose `typedBy` is an element-type name (`PartDef`, `Requirement`, …) holds an id or qualified name, e.g. `reviewer: User::jane`, `goal: SG-GTI-001`.
- A reference that does not resolve, or resolves to an element of a type the feature does not allow, is `E326`.
- The reference appears in `refs`, `impact` and `links` with the stereotype and key as the link name, and takes part in the delete referrer guard (`mutate::referrers`).
- A reference does not by itself create a suspect-link baseline.
