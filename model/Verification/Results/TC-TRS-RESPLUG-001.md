---
type: TestCase
id: TC-TRS-RESPLUG-001
name: "a results plugin converts a foreign report into normalised records"
status: draft
testLevel: L2
sourceFile: repo:crates/syscribe/tests/results_plugins.rs
verifies:
  - REQ-TRS-RESPLUG-001
tags:
  - results
---

```gherkin
Feature: a results plugin converts a foreign report into normalised records

  Scenario: a configured plugin ingests a report
    Given [plugins.rig] with kind results and a converter script
    When ingest-results --format plugin:rig is run on its log
    Then the records are stored exactly as with --format records

  Scenario: plugin failures are findings
    Given a plugin that is missing, times out, exits non-zero or prints invalid JSON
    Then the ingest fails with a finding naming the alias and nothing is written

  Scenario: a plugin cannot bypass validation
    Given a plugin emitting a waived record with no rationale
    Then the whole ingest is rejected

  Scenario: kinds do not mix
    Given a foreignFormat package naming a results plugin
    Then it is an error, and so is --format plugin: naming a foreign-format plugin

  Scenario: dry run
    When plugins run rig --report log.txt --dry-run is run
    Then the raw envelope is printed and nothing is merged
```
