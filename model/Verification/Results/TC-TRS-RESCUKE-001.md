---
type: TestCase
id: TC-TRS-RESCUKE-001
name: "Cucumber JSON is ingested as a built-in format, one record per scenario"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe/tests/results_cucumber.rs
verifies:
  - REQ-TRS-RESCUKE-001
tags:
  - results
---

```gherkin
Feature: Cucumber JSON ingestion

  Scenario: step statuses roll up to a scenario verdict
    Given scenarios whose steps are all passed, one failed, one pending and one undefined
    Then they are pass, fail, not-run and not-run, and the failure keeps its step text and message

  Scenario: a background and hooks count
    Given a failing background step and a failing after hook
    Then the scenarios fail

  Scenario: outline rows
    Given a Scenario Outline with three example rows, one failing
    Then three records "<title> [row N]" are stored and the outline's own title is fail

  Scenario: joining by tag
    Given scenarios tagged with a TestCase id, with extref:<value>, and with the tag only on the feature
    Then each joins its TestCase, and a scenario titled like a Scenario: block is recorded against it

  Scenario: unmatched scenarios
    Given a scenario with no matching tag
    Then it is counted as unmatched and not dropped

  Scenario: retries
    Given the same scenario twice, failing then passing
    Then the verdict is flaky, and fail when the last attempt fails

  Scenario: other inputs
    Given an NDJSON message-protocol file and a file that is not Cucumber output
    Then each is rejected with a message naming what was found and nothing is written

  Scenario: the three implementations
    Given real Cucumber.js 11.3.0 and Cucumber-Ruby 3.1.2 reports and a JVM-shaped one, of the same suite
    Then their verdicts, steps, outline rows and joined TestCases agree, including a hook failing in each

  Scenario: a scenario with no steps of its own
    Given a scenario whose background passed but which has no steps
    Then it is not-run, not pass

  Scenario: attempts and rows
    Given outline rows sharing an id and differing by line, and repeated attempts of one scenario
    Then the rows are numbered and the attempts fold by the last one

  Scenario: provenance
    Given start timestamps and nanosecond durations
    Then startedAt keeps its zone and finishedAt adds the background, steps and hooks

  Scenario: detection and the worked example
    Given a .json report with no --format and examples/results-cucumber
    Then it is read as Cucumber and the example behaves as its README says

  Scenario: attachments are not stored
    Given a report with a large embedded screenshot
    Then the sidecar holds no embedded data
```
