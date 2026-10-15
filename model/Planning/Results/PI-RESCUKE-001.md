---
type: PlanningItem
id: PI-RESCUKE-001
name: "Cucumber JSON as a built-in format"
status: done
itemType: feature
achieves: [REQ-TRS-RESCUKE-001]
evidence:
  - ref: TC-TRS-RESCUKE-001
  - path: repo:crates/syscribe/tests/results_cucumber.rs
  - path: repo:crates/syscribe-model/src/result_cucumber.rs
  - path: repo:examples/results-cucumber/README.md
tags:
  - results
---

`result_cucumber.rs` (a serde_json read of the classic formatter; handles the three implementations' differences in backgrounds, hooks, outline rows and retries; scenarios keyed by feature, id and line; `to_records` joins by tag), `result_robot::add_seconds_zoned`, `ingest-results --format cucumber` with detection by shape, and the NDJSON message protocol refused by name.

Tested against real Cucumber.js 11.3.0 and Cucumber-Ruby 3.1.2 reports of one suite (`crates/syscribe/tests/fixtures/cucumber/`, with the feature) and a hand-written JVM-shaped report; Cucumber-JVM could not be run here, and the README says so.

Documentation done: the ingest help, the CLI reference, the spec, `docs/guides/tests.md`, `CLAUDE.md` and `examples/results-cucumber/`. No new validation codes.

Deviations from the first draft of the requirement, now recorded in it: a scenario with no steps of its own is `not-run` even if its background passed; the error detail keeps the first line only; attempts are keyed by feature, id and line; a `.json` report is recognised by its shape.
