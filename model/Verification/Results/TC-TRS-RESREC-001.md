---
type: TestCase
id: TC-TRS-RESREC-001
name: "normalised result records carry verdict, measurements and provenance"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe/tests/results_records.rs
verifies:
  - REQ-TRS-RESREC-001
tags:
  - results
---

```gherkin
Feature: normalised result records carry verdict, measurements and provenance

  Scenario: a valid record file is ingested
    Given a records file with pass, fail, blocked, not-run, inconclusive and waived verdicts
    Then ingest-results stores them and trace shows [waived] distinct from [pass]

  Scenario: malformed records fail closed
    Given a record with an unknown verdict, a waived verdict without rationale, a bad hash or an inconsistent measurement
    Then ingest-results exits non-zero naming the record and writes nothing

  Scenario: measurements are checked with units
    Given a pass record whose measurement 6 ms violates its limit < 5 ms
    Then the ingest is rejected, and a limit in the wrong dimension is rejected too

  Scenario: records from another rig do not erase this one
    Given bench records ingested, then hil records ingested
    Then both are kept, and with --replace the absent ones are dropped

  Scenario: a repeated key in one file keeps the last record
    Given a file with a failing and then a passing record for one test, scenario and environment
    Then one record is stored, the passing one, and the ingest reports the superseded count

  Scenario: odd but legitimate input
    Given a file with a byte-order mark, null optional fields, unicode scenario titles, or 5000 records
    Then each is ingested, and a non-UTF-8 file is a clean error

  Scenario: the old consumers are unchanged
    Given a records ingest
    Then trace, matrix, safety-case, W010 and W615 read the derived verdicts as before
```
