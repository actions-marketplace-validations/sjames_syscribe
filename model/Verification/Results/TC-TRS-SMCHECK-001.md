---
type: TestCase
id: TC-TRS-SMCHECK-001
name: "a state machine is model-checked over a declared finite environment"
status: draft
testLevel: L2
sourceFile: repo:crates/syscribe/tests/model_check.rs
verifies:
  - REQ-TRS-SMCHECK-001
tags:
  - model-checking
---

```gherkin
Feature: model checking

  Scenario: a violated invariant has a shortest replayable counterexample
    Given a machine whose invariant fails after three events
    Then model-check reports violated with a three-step script and the script reproduces the violation in simulate

  Scenario: proved only when exhausted
    Given a machine whose reachable space fits the bounds
    Then the verdict is proved, and with a smaller --max-states it is inconclusive, never proved

  Scenario: built-in checks
    Given a machine with a dead state, a deadlock, a run error and a cross-region tie
    Then each is reported with the path that reaches it

  Scenario: undeclared input is an error
    Given an input variable with no finite domain
    Then model-check fails naming it and explores nothing

  Scenario: exit codes fail closed
    Then proved exits 0, violated 1, a model error 2, and inconclusive is non-zero
```
