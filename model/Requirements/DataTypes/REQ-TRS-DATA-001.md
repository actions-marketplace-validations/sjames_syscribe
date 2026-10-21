---
type: Requirement
id: REQ-TRS-DATA-001
name: "An AttributeDef constrains its values"
status: draft
reqDomain: software
reqClass: system
tags:
  - data-types
---

An `AttributeDef` shall be able to constrain the values of the type it defines (GH #366, item 2).

## Behavior

- `range:` is a numeric interval: `lo..hi` (closed), or interval notation `[lo, hi]`, `(lo, hi)`, `[lo, hi)`, `(lo, hi]` with a round bracket for an open end. `pattern:` is a regular expression a string value must match in full. `default:` is the value taken when none is given.
- A `range:` that does not parse (or has `lo > hi`), or a `pattern:` that is not a valid regular expression, is `E680`.
- The fields are recognised on an `AttributeDef`; on any other element type they are `W047`.
- A typedef is an `AttributeDef` with a `supertype:`; a struct is an `ItemDef` whose features are attributes. Neither needs a new element type.
