---
type: TestCase
id: TC-TRS-META-022
name: "Applications are exported as structured data"
status: draft
testLevel: L2
sourceFile: repo:crates/syscribe/tests/metadata_query.rs
verifies:
  - REQ-TRS-META-022
tags:
  - metadata
---

```gherkin
Feature: Applications are exported as structured data

  Scenario: show --json
    Then applications are listed as {def, values} with defaulted values marked

  Scenario: export and MCP
    Then the same structure is returned

  Scenario: no applications
    Then no empty key is added to the output
```
