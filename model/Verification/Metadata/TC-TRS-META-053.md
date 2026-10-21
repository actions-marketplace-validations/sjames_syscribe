---
type: TestCase
id: TC-TRS-META-053
name: "Selectors and thresholds work in rules, views and coverage"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe/tests/coverage_gate.rs
verifies:
  - REQ-TRS-META-053
tags:
  - metadata
---

```gherkin
Feature: Selectors and thresholds work in rules, views and coverage

  Scenario: an architecture rule is scoped by a stereotype selector
    Then it chooses only the components that apply the stereotype

  Scenario: a coverage rule is chosen by a stereotype selector
    Then [[coverage.rule]] stereotype picks the rule for the requirements that apply it

  Scenario: a threshold below the percentage
    Given [[gates.coverage]] selecting Safety where integrity >= 2
    Then validate lists the incomplete requirements and exits 2

  Scenario: the threshold is met
    Then validate exits 0

  Scenario: malformed or empty gates
    Then they are usage errors, exit 1
```
