---
type: TestCase
id: TC-TRS-META-032
name: "A standard stereotype library ships with the tool"
status: draft
testLevel: L2
sourceFile: repo:crates/syscribe-model/tests/metadata_profiles.rs
verifies:
  - REQ-TRS-META-032
tags:
  - metadata
---

```gherkin
Feature: A standard stereotype library ships with the tool

  Scenario: resolves
    Then Safety, Optional, Regulatory, External, Environment, Deployment and Budget resolve without E317

  Scenario: visible
    Then they are listed by types and list

  Scenario: typed
    Then an invalid Safety level is reported per the typing rules

  Scenario: extended
    Then a project definition can specialise one

  Scenario: read-only
    Then a write route that edits or deletes one is refused
```
