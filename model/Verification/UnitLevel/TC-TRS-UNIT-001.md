---
type: TestCase
id: TC-TRS-UNIT-001
name: "A software part declares its design level"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe-model/tests/unit_level.rs
verifies:
  - REQ-TRS-UNIT-001
tags:
  - unit-level
---

```gherkin
Feature: A software part declares its design level

  Scenario: checks
    Then the findings of the requirement are raised
```
