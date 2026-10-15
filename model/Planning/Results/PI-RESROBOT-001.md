---
type: PlanningItem
id: PI-RESROBOT-001
name: "Robot Framework output.xml as a built-in format"
status: done
itemType: feature
achieves: [REQ-TRS-RESROBOT-001]
evidence:
  - ref: TC-TRS-RESROBOT-001
  - path: repo:crates/syscribe/tests/results_robot.rs
  - path: repo:crates/syscribe-model/src/result_robot.rs
  - path: repo:examples/results-robot/README.md
tags:
  - results
---

`result_robot.rs` (a streaming quick-xml parse that tracks depth so only a test's own `<status>` counts; Robot 6 and 7 time formats; `to_records` joins by tag against the model), `from_records` and `validate_all` so the adapter's records pass the same validator, `provenance.tool`, `ingest-results --format robot` with detection by root element, and the flags `--operator`, `--artefact` (blake3 file hash) and `--utc-offset`.

Tested against real files from Robot 7.4.2 and 6.1.1 (`crates/syscribe/tests/fixtures/robot/`) and a live run when `robot` is installed.

Documentation done: the ingest help, the CLI reference, the spec (Robot paragraph and `provenance.tool`), `docs/guides/tests.md`, `CLAUDE.md` and `examples/results-robot/`. No new validation codes.

Deviations from the first draft of the requirement, now recorded in it: a test under a failed parent suite setup is `blocked`, not `fail`; Robot's zone-less times take a stated `--utc-offset`; a test with two matching tags records for both TestCases.
