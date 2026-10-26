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

- **Input and detection.** A `.json` file whose first element is an array of features with `elements` and `keyword` is read as Cucumber when no `--format` is given (anything else stays `cargo-json`). The classic JSON formatter output: an array of features, each with `elements` of type `scenario` (and `background`), steps with a `result.status`, and `before`/`after` hooks. The newer NDJSON message protocol is not read; a file that looks like one is rejected with a message that says so, and any other file is a hard error naming what was found.
- **One record per scenario.** A background's steps and hooks count toward every scenario that follows it, whether it is a separate `background` element (JVM, Ruby: the most recent one applies) or merged into each scenario's steps (JS); a failing background step or hook fails the scenario. Scenario Outline rows arrive as one element per row and are recorded as `<title> [row N]` in report order, the convention `simulate` already uses; the outline's own title carries the worst of its rows.
- **Roll-up of a scenario.** Any `failed` or `ambiguous` step, or a failed hook, makes it `fail`, keeping the first failing step's text and the first line of its `error_message` (a stack trace is not kept) as the detail. Otherwise any `pending`, `undefined` or `skipped` step makes it `not-run`; a scenario with no steps *of its own* is `not-run`, whatever its background did; all `passed` is `pass`. A `skipped` step after a failure is a consequence, not a separate finding. A hook is a `before`/`after` array of an element (JVM, Ruby) or a step with the keyword `Before`/`After` (JS); all count.
- **Reruns.** The same scenario is the same feature, element `id` and `line` (outline rows share an `id` in JS and differ by `line`; in JVM and Ruby they differ by `id`). If it appears more than once (a retry plugin writes every attempt) and the last attempt passes after an earlier failure, the verdict is `flaky`, not `pass`, with the failed attempt's detail; if the last attempt fails it is `fail`. Cucumber.js writes only the last attempt of a retry, so it can never show a flaky scenario.
- **Joining.** A scenario is joined to a TestCase by a **tag**, taken from the scenario or inherited from its feature, equal to a TestCase id (`@TC-SYS-012`) or `@extref:<value>` (REQ-TRS-RESJOIN-001). If the scenario's title equals a `Scenario:` title of that TestCase the record is against that scenario, otherwise it is the TestCase's own verdict. A scenario with no matching tag is counted and reported as unmatched, never dropped.
- **Several scenarios, one TestCase.** All must pass; the worst verdict wins and the first failing scenario's title and detail are kept.
- **Provenance.** `startedAt` from the element's `start_timestamp` when the report has one (kept with its zone), and `finishedAt` that plus the durations of the last attempt's steps, background and hooks; a report with no timestamps gets none invented. The report carries no build, rig or operator: they come from `--build-commit`, `--environment`, `--operator` and `--artefact` (REQ-TRS-RESPLUG-001, command-line values).
- **Not read or stored.** Embedded attachments (screenshots, base64 `embeddings`) and step output are never stored; a file path may be passed as an artefact and is referenced by hash.

## Documentation and spec (part of the acceptance of this requirement)

The `ingest-results` help page and CLI reference (formats table and the Cucumber mapping); `docs/guides/tests.md` (tagging scenarios so they join, outline rows, reruns); a worked example `examples/results-cucumber/` with a sample report from each of the three implementations; the spec results chapter; both validation catalogues for any new code; and the drift tests.
