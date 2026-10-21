---
type: Requirement
id: REQ-TRS-META-021
name: "--metadata accepts several definitions and negation"
status: draft
reqDomain: software
reqClass: system
tags:
  - metadata
---

`--metadata` shall take several definitions and a negated form (GH #376).

## Behavior

- `--metadata A --metadata B` keeps elements that apply both; a comma list keeps elements that apply either. `--metadata '!A'` keeps elements that do not apply `A`.
- Matching follows specialisation (REQ-TRS-META-030).
- An unknown definition name is reported, not silently empty.
