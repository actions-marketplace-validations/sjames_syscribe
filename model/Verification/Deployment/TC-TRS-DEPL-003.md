---
type: TestCase
id: TC-TRS-DEPL-003
name: "A node must be at least as trustworthy as what it hosts"
status: draft
testLevel: L2
sourceFile: repo:crates/syscribe-model/tests/deployment.rs
verifies:
  - REQ-TRS-DEPL-003
tags:
  - deployment
---

```gherkin
Feature: A node must be at least as trustworthy as what it hosts

  Scenario: lower node
    Then E733

  Scenario: undeclared
    Then no finding
```
