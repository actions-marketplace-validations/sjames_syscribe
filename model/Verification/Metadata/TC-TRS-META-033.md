---
type: TestCase
id: TC-TRS-META-033
name: "A profile may apply stereotypes automatically"
status: draft
testLevel: L2
sourceFile: repo:crates/syscribe-model/tests/metadata_profiles.rs
verifies:
  - REQ-TRS-META-033
tags:
  - metadata
---

```gherkin
Feature: A profile may apply stereotypes automatically

  Scenario: derived application
    Given a rule: PartDef with domain software and isDeploymentPackage gets Deployment
    Then a matching part has the application in queries, checks and diagrams, marked derived in show

  Scenario: never written
    Then the part's file is unchanged

  Scenario: explicit wins
    Given the part applies Deployment explicitly
    Then the explicit values are used
```
