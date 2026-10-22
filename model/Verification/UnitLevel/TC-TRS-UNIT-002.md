---
type: TestCase
id: TC-TRS-UNIT-002
name: "A software unit is traced to code, requirement, test and component"
status: draft
testLevel: L2
sourceFile: repo:crates/syscribe-model/tests/unit_level.rs
verifies:
  - REQ-TRS-UNIT-002
tags:
  - unit-level
---

```gherkin
Feature: A software unit is traced to code, requirement, test and component

  Scenario: checks
    Then the findings of the requirement are raised
```
