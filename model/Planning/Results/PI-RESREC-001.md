---
type: PlanningItem
id: PI-RESREC-001
name: "Normalised result records: verdicts, measurements, provenance"
status: done
itemType: feature
achieves: [REQ-TRS-RESREC-001]
evidence:
  - ref: TC-TRS-RESREC-001
  - path: repo:crates/syscribe/tests/results_records.rs
  - path: repo:crates/syscribe-model/src/result_records.rs
tags:
  - results
---

Requirement and test case first, then the tests, then the implementation (`result_records.rs`, the `records`/`by_record` sidecar sections, `Verdict::{Blocked,NotRun,Inconclusive,Waived}`, `safety_case::Verdict::Waived`, `ingest-results --format records [--replace]`), then the documentation the requirement lists: the `ingest-results` help page, the CLI reference, the authoring prompt, the format spec, `docs/guides/tests.md` and `examples/results-records/`. No new validation codes, so neither catalogue changed.

Not part of this item, by design: joining by `extRef` and the stale-result warning (REQ-TRS-RESJOIN-001), results plugins (REQ-TRS-RESPLUG-001), per-environment plan verdicts (REQ-TRS-RESENV-001), and the `template TestCase` hints, which belong with the features they describe.
