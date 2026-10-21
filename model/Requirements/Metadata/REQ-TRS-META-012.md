---
type: Requirement
id: REQ-TRS-META-012
name: "Tagged-value defaults are visible without being written"
status: draft
reqDomain: software
reqClass: system
tags:
  - metadata
---

An omitted tagged value shall take the default declared on its feature, in every consumer (GH #375).

## Behavior

- `default` on a `MetadataDef` feature supplies the value of an application that omits the key. A feature with a default is never `E325` for omission.
- The default is visible in `show` (marked `(default)`) and in `list --json`, under `metadata` as `{def, values, defaulted}`; `list --where meta...` and `meta(Def).key` follow with REQ-TRS-META-020 and -023.
- A default is checked against the feature's own type, unit, range and enumeration; a violating default is `E322`/`E323`/`E324` on the definition.
- An explicit value always wins; the file is never rewritten to materialise a default.
