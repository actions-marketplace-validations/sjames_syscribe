---
type: TestCase
id: TC-TRS-BUD-003
name: "A budgets report shows use against capacity"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe/tests/budgets_cli.rs
verifies:
  - REQ-TRS-BUD-003
tags:
  - budgets
---

```gherkin
Feature: A budgets report shows use against capacity

  Scenario: json and text
    Then used, capacity, limit and contributors are reported largest first

  Scenario: project margin
    Then [budgets] margin applies unless the part sets its own

  Scenario: nothing declared
    Then the report says so
```
