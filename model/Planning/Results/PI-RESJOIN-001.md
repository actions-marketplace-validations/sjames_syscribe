---
type: PlanningItem
id: PI-RESJOIN-001
name: "Join results by id or extRef, report stale results"
status: done
itemType: feature
achieves: [REQ-TRS-RESJOIN-001]
evidence:
  - ref: TC-TRS-RESJOIN-001
  - path: repo:crates/syscribe/tests/results_join.rs
  - path: repo:crates/syscribe-model/src/result_join.rs
tags:
  - results
---

Join by id, qualified name, then `extRef` (`result_join.rs`); the canonical id is what is stored. Ambiguous or non-TestCase keys reject the file; keys naming nothing go to `unmatched` (listed by `results failures`, `--strict` rejects). Each record keeps the projection hash of its TestCase and of the requirements it verifies; `validate` raises `W091` for a non-draft TestCase whose results predate a change, `trace` marks it `[stale]`.

Documentation done: both validation catalogues and the index range table, the spec (results chapter, `extRef`, code table), `ingest-results` help, the CLI reference, `docs/guides/tests.md`, `docs/design/suspect-links.md`, and the `template TestCase` skeleton (commented `extRef:` line).

Not done, by design: the `matrix` grid is not annotated (the finding is its signal), and `RESENV` will decide how records of several rigs are judged by a plan.
