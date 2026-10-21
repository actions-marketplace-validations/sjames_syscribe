---
type: TestCase
id: TC-TRS-ARCH-005
name: "independence reports the shared element"
status: draft
testLevel: L2
sourceFile: repo:crates/syscribe-model/tests/arch_rules.rs
verifies:
  - REQ-TRS-ARCH-005
tags:
  - architecture-rules
---

```gherkin
Feature: independence reports the shared element

  Scenario: two chains sharing a component
    Then the shared component is reported with a path from each side

  Scenario: removing the sharing
    Then the rule passes

  Scenario: direct wiring
    Then the wired-to component is shared
```
