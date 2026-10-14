---
type: TestCase
id: TC-TRS-RESROBOT-001
name: "Robot Framework output.xml is ingested as a built-in format"
status: draft
testLevel: L2
sourceFile: repo:crates/syscribe/tests/results_robot.rs
verifies:
  - REQ-TRS-RESROBOT-001
tags:
  - results
---

```gherkin
Feature: Robot Framework output.xml is ingested as a built-in format

  Scenario: both Robot versions
    Given an output.xml from Robot 6 and one from Robot 7
    Then both are read, with their timestamp formats

  Scenario: verdict mapping
    Given PASS, FAIL, SKIP and NOT RUN tests
    Then they map to pass, fail, not-run and not-run, a FAIL keeping its message

  Scenario: joining by tag
    Given tests tagged with a TestCase id or extref:<value>
    Then they join, and a test named like a Scenario title is recorded against it

  Scenario: untagged tests
    Given a test with no matching tag
    Then it is counted as unmatched and not dropped

  Scenario: several tests, one TestCase
    Given two tests for one TestCase, one failing
    Then the TestCase fails and keeps the first failing test's name and message

  Scenario: not a Robot file
    Given an XML file with another root element
    Then the ingest fails naming the root and writes nothing
```
