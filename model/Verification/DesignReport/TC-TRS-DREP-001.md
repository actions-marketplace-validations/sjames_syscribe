---
type: TestCase
id: TC-TRS-DREP-001
name: "A design report lays out one software part"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe/tests/design_report_cli.rs
verifies:
  - REQ-TRS-DREP-001
tags:
  - unit-level
---

```gherkin
Feature: A design report lays out one software part

  Scenario: the report
    Then it lists interface, behaviour, data, resources, code, requirements and tests
```
