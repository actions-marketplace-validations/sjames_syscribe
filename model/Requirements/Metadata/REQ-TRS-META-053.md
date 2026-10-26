---
type: Requirement
id: REQ-TRS-META-053
name: "Selectors and thresholds work in rules, views and coverage"
status: draft
reqDomain: software
reqClass: system
tags:
  - metadata
---

Stereotype selectors shall be accepted by architecture rules, views and coverage, and a coverage threshold shall be settable per stereotype (GH #379, #369).

The selector grammar, the TestPlan `selection.stereotypes:` and `matrix --select` (REQ-TRS-META-024), a diagram view's `include: {stereotypes: […]}`, the `stereotype` selector of `[[coverage.rule]]`, and the `stereotype` selector of an `ArchitectureRule` (REQ-TRS-ARCH-002) are done. This requirement adds the threshold.

## Behavior

- A `[[coverage.rule]]` may select requirements by `stereotype`, so a stereotype can choose how its parents are judged complete. A rule that loosens an integrity-rated parent is still `E898`.
- `[[gates.coverage]]` in `.syscribe.toml` sets a threshold for the requirements a stereotype selector chooses: `select` (a selector string, `"Safety where integrity >= 2"`; a specialisation counts) and `min_complete` (a percentage, 0–100, default 100).
- The measure is the roll-up verdict of `matrix --rollup` (`complete`). Draft requirements and those whose verdict is `na` are not counted. The percentage is complete over counted.
- `validate` reports each gate below its threshold — the selector, `n/m complete`, the percentage against the threshold and the incomplete requirements — and exits 2 (a tripped gate); errors still exit 1.
- A malformed entry (unknown key, a percentage outside 0–100, a malformed selector or one naming no `MetadataDef`) or one that selects no counted requirement is a usage error, exit 1.
