---
type: TestCase
id: TC-TRS-BUD-006
name: "CPU utilisation of what is allocated to a processor is checked against a bound"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe-model/tests/budgets_cpu.rs
verifies:
  - REQ-TRS-BUD-006
tags:
  - budgets
---

```gherkin
Feature: CPU utilisation of what is allocated to a processor is checked against a bound

  Scenario: edf bound
    Then utilisation above cpuMargin percent is E692 naming the tasks

  Scenario: rate-monotonic bound
    Then the Liu-Layland bound applies

  Scenario: containment and allocation
    Then contained tasks count once

  Scenario: opt in
    Then a part with no cpuMargin or scheduling is not checked

  Scenario: report
    Then budgets --json carries cpu with the tasks
```
