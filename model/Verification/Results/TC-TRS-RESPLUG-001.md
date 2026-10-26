---
type: TestCase
id: TC-TRS-RESPLUG-001
name: "a results plugin converts a foreign report into normalised records"
status: active
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

  Scenario: the protocol
    Given a plugin that saves its stdin
    Then the request has protocolVersion, alias, absolute reportPath and modelRoot, and run, config and environment only when given

  Scenario: command-line values fill, never override
    Given a plugin whose records leave the rig and the commit empty, and one that sets its own commit
    Then --environment and --build-commit fill the gaps and the record's own commit is kept

  Scenario: robustness
    Given a plugin that ignores its stdin, writes 5000 records, hangs, or exits zero with no output
    Then each is handled without a deadlock, within the timeout, and as an error where it is one

  Scenario: the worked example
    Given examples/stdio-plugins/results-python
    Then its rig and TAP converters ingest as the README says

  Scenario: dry run
    When plugins run rig --report log.txt --dry-run is run
    Then the raw envelope is printed and nothing is merged
```
