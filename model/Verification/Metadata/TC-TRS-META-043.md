---
type: TestCase
id: TC-TRS-META-043
name: "The diagram legend lists the styles used"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe-model/tests/vis_stereotype.rs
verifies:
  - REQ-TRS-META-043
tags:
  - metadata
---

```gherkin
Feature: The diagram legend lists the styles used

  Scenario: exact legend
    Then the legend lists exactly the stereotypes that styled something

  Scenario: nothing styled
    Then no legend is emitted
```
