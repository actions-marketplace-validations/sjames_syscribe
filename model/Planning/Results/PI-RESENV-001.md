---
type: PlanningItem
id: PI-RESENV-001
name: "Results per TestEnvironment, plans judge their rigs"
status: done
itemType: feature
achieves: [REQ-TRS-RESENV-001]
evidence:
  - ref: TC-TRS-RESENV-001
  - path: repo:crates/syscribe/tests/results_env.rs
  - path: repo:crates/syscribe-model/src/result_join.rs
tags:
  - results
---

`ResultsData::view_for_env` (only the records of one rig, none from an unhealthy one), `testcase_verdict_in_envs` (every listed rig must pass), used by `plan_verdict`, by `tc_verdict` for a TestCase's own `runsOn:`, and by the `W615` extension; `ingest-results --environment`; `trace` and `results failures` name the rig.

Documentation done: the testplan guide and help, the ingest help, the CLI reference, the spec (results per TestEnvironment), both `W615` rows, the `TestCase` and `TestPlan` skeletons (commented `runsOn:`), and the worked example (a plan over both rigs).

Known limits, stated in the requirement: a TestCase's own `runsOn:` is matched as written (use ids); rig health is the rig's state now; `testplan` takes no `--config` in its argument table although the lens list names it (pre-existing); `results diff` is not per rig.
