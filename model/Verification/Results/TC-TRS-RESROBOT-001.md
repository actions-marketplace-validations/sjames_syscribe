---
type: TestCase
id: TC-TRS-RESROBOT-001
name: "Robot Framework output.xml is ingested as a built-in format"
status: active
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

  Scenario: setup failures
    Given a suite whose setup fails, and tests whose own setup or teardown fails
    Then the first are blocked and the others fail

  Scenario: nested suites and special characters
    Given nested suites and a test name and message with markup characters
    Then the full name is kept and the entities are decoded

  Scenario: provenance from the command line
    Given --environment, --build-commit, --operator, --artefact and --utc-offset
    Then they fill the records, the artefact is referenced by its hash and never copied, and a bad offset is rejected

  Scenario: detection
    Given an xml file with a robot root and one with a testsuite root and no --format
    Then the first is read as Robot and the second as JUnit

  Scenario: the worked example and a live run
    Given examples/results-robot and, when installed, a fresh run of Robot
    Then both ingest as described

  Scenario: not a Robot file
    Given an XML file with another root element
    Then the ingest fails naming the root and writes nothing
```
