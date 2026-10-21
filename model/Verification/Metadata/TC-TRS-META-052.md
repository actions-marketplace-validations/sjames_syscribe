---
type: TestCase
id: TC-TRS-META-052
name: "A meta report groups elements by definition and value"
status: draft
testLevel: L2
sourceFile: repo:crates/syscribe-model/tests/metadata_checks.rs
verifies:
  - REQ-TRS-META-052
tags:
  - metadata
---

```gherkin
Feature: A meta report groups elements by definition and value

  Scenario: group by
    Given the grid-tie model
    When meta --def Regulatory --group-by status
    Then the regulatory elements are listed by verification status

  Scenario: json
    Then --json gives the same data

  Scenario: defaults and inheritance
    Then they are included
```
