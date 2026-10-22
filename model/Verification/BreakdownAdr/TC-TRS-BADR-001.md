---
type: TestCase
id: TC-TRS-BADR-001
name: "The breakdown ADR requirement can be disabled"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe-model/tests/breakdown_adr.rs
verifies:
  - REQ-TRS-BADR-001
tags:
  - traceability
---

```gherkin
Feature: The breakdown ADR requirement can be disabled

  Scenario: default
    Given a requirement with derivedFrom and no breakdownAdr
    Then it is E310

  Scenario: disabled
    Given require_breakdown_adr = false
    Then there is no E310

  Scenario: a cited ADR is still checked
    Given require_breakdown_adr = false and a breakdownAdr that does not resolve
    Then it is E311

  Scenario: malformed
    Given require_breakdown_adr = "no"
    Then it is W632 and E310 still applies
```
