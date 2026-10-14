---
type: Requirement
id: REQ-TRS-RESROBOT-001
name: "ingest-results reads Robot Framework output.xml as a built-in format"
status: draft
reqDomain: software
reqClass: system
tags:
  - results
---

`ingest-results --format robot <output.xml>` shall convert a Robot Framework result file into normalised result records (REQ-TRS-RESREC-001), so system and acceptance tests written in Robot join the traceability graph without a converter.

## Behavior

- **Which Robot.** Robot Framework 6 and 7 `output.xml` (the `<robot>` root with nested `<suite>` and `<test>`). The timestamp format differs between the two (`20240131 12:00:00.000` and ISO 8601) and both are read. A file that is not a Robot result is a hard error naming the root element.
- **Verdicts.** `PASS` is `pass`; `FAIL` is `fail`, keeping the `message` as a one-line detail like JUnit; `SKIP` and `NOT RUN` are `not-run`. A test whose setup failed is `fail`. A suite-level failure does not mark the tests it did not run as failed.
- **Joining.** A Robot test is joined to a TestCase by a **tag** equal to a TestCase id, or `extref:<value>` for an `extRef` (REQ-TRS-RESJOIN-001). A test whose name equals a `Scenario:` title of that TestCase is recorded against the scenario; otherwise the record is the TestCase's own verdict. A test with no matching tag is counted and reported as unmatched, never dropped.
- **Several tests, one TestCase.** All must pass; the worst verdict wins, and the first failing test's name and message are kept.
- **Provenance.** `startedAt`/`finishedAt` from the test's status element; the Robot version from the `generator` attribute is recorded as the producing tool. Build, environment and operator are not in the file: they come from `--build-commit`, `--environment` and `--operator` (REQ-TRS-RESPLUG-001, command-line values).
- **Not read.** Keyword-level results, logs and screenshots; a screenshot or log path may be passed as an artefact with `--artefact <uri>` and is referenced by hash, never copied.

## Documentation and spec (part of the acceptance of this requirement)

The `ingest-results` help page and CLI reference (formats table and the Robot mapping); `docs/guides/tests.md` (writing Robot tests that join: tags, `extref:`); a small `examples/results-robot/` with a sample `output.xml` for both Robot versions; the spec results chapter; and the drift tests. Cucumber JSON is a separate built-in format (REQ-TRS-RESCUKE-001); TAP is not built in and ships as an example plugin (REQ-TRS-RESPLUG-001).
