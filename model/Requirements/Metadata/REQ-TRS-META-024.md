---
type: Requirement
id: REQ-TRS-META-024
name: "A common selector grammar names elements by stereotype"
status: draft
reqDomain: software
reqClass: system
tags:
  - metadata
---

A selector `{stereotype: Def, where: key>=2}` shall be usable wherever the tool selects elements (GH #376).

## Behavior

- The selector selects elements applying `Def` (with specialisation) whose tagged values satisfy `where`.
- It is written as a mapping or as the string `Def where <expr>`, parsed by one function. It is accepted in a TestPlan's `selection.stereotypes:` (a TestCase is selected when it, or a requirement it verifies, matches) and by `matrix --select` (repeatable, ANDed). Diagram `include:` and `ArchitectureRule` scopes (#334, #369) will use it when they exist.
- A malformed selector, or one naming no `MetadataDef`, is `E651` on the TestPlan, and a usage error (exit 1) for `matrix --select`.
