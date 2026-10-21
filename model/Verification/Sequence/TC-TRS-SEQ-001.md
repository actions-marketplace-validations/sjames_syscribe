---
type: TestCase
id: TC-TRS-SEQ-001
name: "A part is a sequence subject through the actions it performs"
status: draft
testLevel: L2
sourceFile: repo:crates/syscribe-model/tests/vis_derive_sequence.rs
verifies:
  - REQ-TRS-SEQ-001
tags:
  - diagram
  - sequence
---

```gherkin
Feature: A part is a sequence subject through the actions it performs

  Scenario: a part that performs an action
    Then its sequence carries that action's messages and the part is the first lifeline

  Scenario: two performed actions
    Then they appear in the order performs: declares

  Scenario: a usage
    Then it uses its definition's performs

  Scenario: nothing performed
    Then W418 names performs: and the diagram is empty
```
