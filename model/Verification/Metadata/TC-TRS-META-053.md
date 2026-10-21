---
type: TestCase
id: TC-TRS-META-053
name: "Selectors and thresholds work in rules, views and coverage"
status: draft
testLevel: L2
sourceFile: repo:crates/syscribe-model/tests/metadata_checks.rs
verifies:
  - REQ-TRS-META-053
tags:
  - metadata
---

```gherkin
Feature: Selectors and thresholds work in rules, views and coverage

  Scenario: rule scope
    Then an ArchitectureRule scoped by a stereotype selector applies only to its carriers

  Scenario: threshold
    Given a coverage threshold higher for Safety
    Then a Safety element below it is reported and others are held to the default
```
