---
type: TestCase
id: TC-TRS-RESENV-001
name: "results are kept per TestEnvironment and a plan judges the rigs it runs on"
status: draft
testLevel: L2
sourceFile: repo:crates/syscribe/tests/results_env.rs
verifies:
  - REQ-TRS-RESENV-001
tags:
  - results
---

```gherkin
Feature: results are kept per TestEnvironment and a plan judges the rigs it runs on

  Scenario: pass on one rig, fail on another
    Given a test passing on the bench and failing on the hil rig
    Then both verdicts are kept and shown as <test> @ <environment>

  Scenario: a plan judges its own rigs
    Given a plan with runsOn the hil rig only
    Then the bench result is ignored and the hil result decides

  Scenario: a missing rig is missing
    Given a plan with runsOn two rigs and a record on one
    Then the other counts as missing and an approved plan raises W615

  Scenario: no runsOn is conservative
    Given a plan without runsOn
    Then a failure on any rig fails it

  Scenario: rig health
    Given a record on a retired or out-of-calibration rig
    Then it is not a pass for a non-draft plan and W892 is raised

  Scenario: configuration first, environment inside
    Given evidence per Configuration and per environment
    Then the configuration's effective results are chosen first and the environment filter applies inside them
```
