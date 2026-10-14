---
type: TestCase
id: TC-TRS-RESJOIN-001
name: "results join by id or extRef, and stale results are reported"
status: draft
testLevel: L2
sourceFile: repo:crates/syscribe/tests/results_join.rs
verifies:
  - REQ-TRS-RESJOIN-001
tags:
  - results
---

```gherkin
Feature: results join by id or extRef, and stale results are reported

  Scenario: join order
    Given records naming a TestCase by id, by qualified name and by extRef
    Then all three attach to the right TestCase

  Scenario: ambiguity and wrong kind
    Given a key matching two elements, or an element that is not a TestCase
    Then the ingest reports it and names the candidates

  Scenario: unmatched records
    Given records whose key matches nothing
    Then they are counted and listed, stored under unmatched, and with --strict the ingest is rejected

  Scenario: staleness
    Given a verdict recorded for a TestCase
    When the TestCase or a requirement it verifies is edited
    Then validate raises the stale-result warning naming what changed, and --deny makes it fail

  Scenario: older sidecars stay silent
    Given a sidecar written without hashes
    Then no stale-result warning is raised
```
