---
type: TestCase
id: TC-TRS-RESENV-001
name: "results are kept per TestEnvironment and a plan judges the rigs it runs on"
status: active
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

  Scenario: recording the rig
    Given records with no environment and ingest-results --environment
    Then the flag fills only the records that name none, an unknown rig is rejected, and the flag is refused for the other formats

  Scenario: a waiver on one rig
    Given a plan with runsOn two rigs and a waived result on one
    Then the plan is incomplete, not passing

  Scenario: a test case with its own runsOn
    Given a TestCase with runsOn the hil rig and a failure on the bench only
    Then it is judged on the hil rig alone

  Scenario: a model that never names a rig
    Given records with no environment and a plan with no runsOn
    Then nothing changes: no marker in trace and no W615

  Scenario: configuration first, environment inside
    Given evidence per Configuration and per environment
    Then the configuration's effective results are chosen first and the environment filter applies inside them
```
