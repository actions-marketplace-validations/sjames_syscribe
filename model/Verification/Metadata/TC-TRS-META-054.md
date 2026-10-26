---
type: TestCase
id: TC-TRS-META-054
name: "A baseline can require a stereotype condition"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe/tests/metadata_checks.rs
verifies:
  - REQ-TRS-META-054
tags:
  - metadata
---

```gherkin
Feature: A baseline can require a stereotype condition

  Scenario: refused
    Given an unverified Regulatory element and a baseline condition against it
    When baseline create runs
    Then it refuses and names the element

  Scenario: allowed
    Then it seals when the condition holds
```
