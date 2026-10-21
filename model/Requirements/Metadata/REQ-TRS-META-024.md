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
- It is accepted in diagram `include:`, `ArchitectureRule` scopes, `matrix`-style filters and test plan `selection:`; each consumer documents that it uses the shared grammar.
- A selector naming an unknown definition or key is reported by the consumer's existing unresolved-reference finding.
