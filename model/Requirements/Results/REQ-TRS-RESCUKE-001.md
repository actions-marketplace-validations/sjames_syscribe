---
type: Requirement
id: REQ-TRS-RESCUKE-001
name: "ingest-results reads Cucumber JSON as a built-in format, one record per scenario"
status: draft
reqDomain: software
reqClass: system
tags:
  - results
---

`ingest-results --format cucumber <report.json>` shall convert the output of the Cucumber `json` formatter (Cucumber-JVM, Cucumber.js and Cucumber-Ruby) into normalised result records (REQ-TRS-RESREC-001). A `TestCase` here already holds Gherkin `Scenario:` blocks, so the scenarios of the report map directly onto the scenarios of the model.

## Behavior

- **Input.** The classic JSON formatter output: an array of features, each with `elements` of type `scenario` (and `background`), steps with a `result.status`, and `before`/`after` hooks. The newer NDJSON message protocol is not read; a file that looks like one is rejected with a message that says so, and any other file is a hard error naming what was found.
- **One record per scenario.** A background's steps count toward every scenario that follows it; a failing background step fails the scenario. Scenario Outline rows arrive as one element per row and are recorded as `<title> [row N]` in report order, the convention `simulate` already uses; the outline's own title carries the worst of its rows.
- **Roll-up of a scenario.** Any `failed` or `ambiguous` step, or a failed hook, makes it `fail`, keeping the first failing step's text and `error_message` (one line, sanitised) as the detail. Otherwise any `pending`, `undefined` or `skipped` step makes it `not-run`; a scenario with no steps is `not-run`; all `passed` is `pass`. A `skipped` step after a failure is a consequence, not a separate finding.
- **Reruns.** If the same scenario appears more than once (a retry plugin) and the last attempt passes after an earlier failure, the verdict is `flaky`, not `pass`; if the last attempt fails it is `fail`.
- **Joining.** A scenario is joined to a TestCase by a **tag**, taken from the scenario or inherited from its feature, equal to a TestCase id (`@TC-SYS-012`) or `@extref:<value>` (REQ-TRS-RESJOIN-001). If the scenario's title equals a `Scenario:` title of that TestCase the record is against that scenario, otherwise it is the TestCase's own verdict. A scenario with no matching tag is counted and reported as unmatched, never dropped.
- **Several scenarios, one TestCase.** All must pass; the worst verdict wins and the first failing scenario's title and detail are kept.
- **Provenance.** `startedAt` from the element's `start_timestamp` when the report has one; the summed step durations as the run time. The report carries no build, rig or operator: they come from `--build-commit`, `--environment`, `--operator` and `--artefact` (REQ-TRS-RESPLUG-001, command-line values).
- **Not read or stored.** Embedded attachments (screenshots, base64 `embeddings`) and step output are never stored; a file path may be passed as an artefact and is referenced by hash.

## Documentation and spec (part of the acceptance of this requirement)

The `ingest-results` help page and CLI reference (formats table and the Cucumber mapping); `docs/guides/tests.md` (tagging scenarios so they join, outline rows, reruns); a worked example `examples/results-cucumber/` with a sample report from each of the three implementations; the spec results chapter; both validation catalogues for any new code; and the drift tests.
