---
type: TestCase
id: TC-TRS-DEPL-002
name: "A component is deployed once unless replicated"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe-model/tests/deployment.rs
verifies:
  - REQ-TRS-DEPL-002
tags:
  - deployment
---

```gherkin
Feature: A component is deployed once unless replicated

  Scenario: two nodes
    Then E732 names the nodes

  Scenario: replicated
    Then no finding
```
