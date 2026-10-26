---
type: Requirement
id: REQ-TRS-META-011
name: "Tagged-value multiplicity makes a value required or a list"
status: draft
reqDomain: software
reqClass: system
tags:
  - metadata
---

A tagged-value feature shall be able to be required, optional or multi-valued through `multiplicity` (GH #375).

## Behavior

- `multiplicity` on a `MetadataDef` feature uses the SysML form (`1`, `0..1`, `1..*`, `2`). An omitted value for a feature whose lower bound is at least 1, and that has no default, is `E325`. Fewer or more values than the bounds is `E325`.
- A feature with an upper bound above 1 accepts a YAML list; a scalar is one value.
- A feature with no `multiplicity` is `0..1`, so existing models are unchanged.
- Draft elements are not exempt: the finding is an error.
