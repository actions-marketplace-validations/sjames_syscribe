---
type: Requirement
id: REQ-TRS-META-052
name: "A meta report groups elements by definition and value"
status: draft
reqDomain: software
reqClass: system
tags:
  - metadata
---

`syscribe meta [--def D] [--group-by key]` shall report counts and lists by stereotype and tagged value (GH #379).

## Behavior

- Output is the compliance worklist: e.g. `Regulatory` grouped by `status`, largest group first, an element without the value in `(unset)`. Without `--def` it counts applications per stereotype. Supports `--json`. An unknown `--def` is a usage error.
- Defaults and inherited values are included.
