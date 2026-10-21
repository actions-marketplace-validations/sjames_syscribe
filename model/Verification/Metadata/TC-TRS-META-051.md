---
type: TestCase
id: TC-TRS-META-051
name: "validate --deny-meta gates on tagged values"
status: draft
testLevel: L2
sourceFile: repo:crates/syscribe-model/tests/metadata_checks.rs
verifies:
  - REQ-TRS-META-051
tags:
  - metadata
---

```gherkin
Feature: validate --deny-meta gates on tagged values

  Scenario: matching element
    Given an element with Regulatory status unverified
    When validate --deny-meta Regulatory.status=unverified
    Then it exits non-zero and lists the element

  Scenario: no match
    Then it exits zero

  Scenario: gates table
    Then the same condition in [gates] fails CI
```
