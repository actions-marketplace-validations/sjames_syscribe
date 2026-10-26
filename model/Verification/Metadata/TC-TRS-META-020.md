---
type: TestCase
id: TC-TRS-META-020
name: "Filter by tagged value with --where meta"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe/tests/metadata_query.rs
verifies:
  - REQ-TRS-META-020
tags:
  - metadata
---

```gherkin
Feature: Filter by tagged value with --where meta

  Scenario: comparison
    Given three parts, two with SafetyCritical integrity 2 and 3, one with 1
    When list PartDef --where meta.SafetyCritical.integrity>=2
    Then exactly the first two are returned

  Scenario: presence
    When --where meta.SafetyCritical.integrity
    Then every element applying SafetyCritical with a value is returned

  Scenario: element without the stereotype
    Then it is not matched and no error is reported

  Scenario: defaults and inherited values
    Then they take part

  Scenario: other commands
    Then find, ls, digest and the MCP tools accept the same expression
```
