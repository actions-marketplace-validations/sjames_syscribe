---
type: Requirement
id: REQ-TRS-META-023
name: "Expressions read stereotypes with meta() and has()"
status: draft
reqDomain: software
reqClass: system
tags:
  - metadata
---

The expression language shall read stereotype applications on the element in scope (GH #376).

## Behavior

- `has(Def)` is true when the element applies `Def` or a specialisation of it. `meta(Def).key` yields the tagged value, or its default.
- On an element that does not apply `Def`, `has(Def)` is false and `meta(Def).key` is an absent value, not an error; comparisons against an absent value are false and `is_absent` style checks can test for it.
- Evaluated against the element in scope, so they are available in `MetadataDef` `constraints:` (REQ-TRS-META-014), where a second stereotype may read the first. `eval` reads no model and does not offer them.
- A call that is malformed, or names a definition that does not resolve, is reported on the definition carrying the constraint.
