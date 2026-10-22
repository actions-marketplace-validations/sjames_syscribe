---
type: TestCase
id: TC-TRS-DEPL-001
name: "Deployment nodes have a kind and nest in order"
status: draft
testLevel: L2
sourceFile: repo:crates/syscribe-model/tests/deployment.rs
verifies:
  - REQ-TRS-DEPL-001
tags:
  - deployment
---

```gherkin
Feature: Deployment nodes have a kind and nest in order

  Scenario: kinds
    Then an unknown kind is E730

  Scenario: nesting
    Then a node inside a node of the same or a later kind is E731

  Scenario: other elements
    Then the field is W047
```
