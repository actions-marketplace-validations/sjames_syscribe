---
type: Requirement
id: REQ-TRS-DREP-001
name: "A design report lays out one software part"
status: draft
reqDomain: software
reqClass: system
tags:
  - unit-level
---

`syscribe design-report <part> [--json]` shall lay out one software part as the detailed-design work product (GH #371).

## Behavior

- Sections: the part's `level` and description; its **interface** (its own `operations:` and the operations of the ports and interfaces it contains, with parameters and return); its **behaviour** (the `ActionDef`s that realise its operations); its **data** (its `features:` that are not ports, with type, unit and, for an `AttributeDef`, `range`); its **resources** (`ram`, `flash`, `nvm`, `stack`, `period`, `deadline`, `wcet`, `priority` as declared); its **code** (`implementedBy:`); the **requirements** it satisfies; and its **tests** (TestCases that verify it or a requirement it satisfies, each with level and verdict).
- A section with nothing to show says so. An argument that is not a software `PartDef`/`Part` is a usage error (exit 1).
- `--json` gives the same content as an object.
