---
type: Requirement
id: REQ-TRS-RESENV-001
name: "Results are kept per TestEnvironment, and a TestPlan judges the environments it runs on"
status: draft
reqDomain: software
reqClass: system
tags:
  - results
  - verification
---

A test that passes on the bench and fails on the hardware-in-the-loop rig shall not collapse into one verdict. The result of a test is kept per `TestEnvironment`, and a `TestPlan` judges the environments it declares in `runsOn`.

## Behavior

- **Recording.** A record's `environment` (REQ-TRS-RESREC-001) or `ingest-results --environment <TE>` names the rig. It must resolve to a `TestEnvironment`, otherwise the ingest is rejected. A record without an environment belongs to the unnamed environment; nothing changes for models that never name one.
- **Latest per key.** The sidecar keeps the latest record per `(testCase, scenario, environment)`; an ingest on one rig does not supersede another rig's results (REQ-TRS-RESREC-001, merge).
- **Plan verdict.** A `TestPlan` or `TestCase` with `runsOn:` is judged on the latest record of each member in **each** listed environment: all must pass, a missing record in a listed environment counts as missing exactly as an absent function does (`W615` for an approved plan), and a record on a rig the plan does not list is ignored by that plan.
- **Without `runsOn:`.** The verdict is the worst over every environment, so a failure on any rig is never hidden. This mirrors the conservative rule of REQ-TRS-CFGRES-001 for configurations.
- **Order with configurations.** A Configuration's effective results are chosen first (REQ-TRS-CFGRES-001), and the environment filter is applied inside them. An environment never selects a configuration, and a configuration never selects an environment.
- **Rig health.** A record on a `retired` environment or one with `calibrationStatus: expired` at the time of the run does not count as a pass for a non-draft plan: it is reported with `W892` as today and listed, not hidden. `calibrationDue` stays advisory.
- **Readers.** `trace`, `matrix`, `safety-case`, `testplan`, `audit`, `results failures` and `results diff` show the environment with a verdict (`<test> @ <environment>`), following the notation already used for configurations.

## Documentation and spec (part of the acceptance of this requirement)

`docs/model-guide/testplans.md` (the `runsOn:` row now also selects results); the help pages `testplan`, `ingest-results` and `results`; the `TestCase` skeleton printed by `syscribe template TestCase` (a commented `runsOn:` line) and the TestPlan skeleton; the spec chapters on `TestEnvironment` and results; both validation catalogues; and the drift tests.
