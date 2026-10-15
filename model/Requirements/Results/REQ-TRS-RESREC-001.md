---
type: Requirement
id: REQ-TRS-RESREC-001
name: "A normalised result record carries verdict, measurements and provenance, and ingest-results accepts it"
status: draft
reqDomain: software
reqClass: system
tags:
  - results
  - evidence
---

`ingest-results` shall accept a **normalised result record**, a superset of the `session-log` record, so that system, integration and hardware-in-the-loop results, which are rarely JUnit, can be ingested with the facts an audit needs: which build, which rig, which measurements, where the raw evidence is.

## Behavior

- **Format.** `ingest-results --format records <file>` reads a JSON array. A record has: `testCase` (a TestCase id, qualified name or `extRef` value; see REQ-TRS-RESJOIN-001); optional `scenario` (a `Scenario:` title of that TestCase; absent means the record is the verdict of the whole TestCase); `verdict`; optional `steps` (opaque, as in session-log; not required); optional `measurements`; and `provenance`.
- **Verdicts.** `pass`, `fail`, `blocked` (could not run, for instance the rig was unavailable), `not-run`, `inconclusive`, and `waived`, which requires a non-empty `rationale`. The existing `ignored` and `flaky` of the function-level formats are unchanged; a session-log `unknown` keeps its meaning. For roll-up, `pass` is a pass; `fail` fails; `flaky`, `blocked`, `not-run`, `inconclusive` and `ignored` are not a pass and leave a TestCase unannotated or flagged as `W010` and `W615` already treat `ignored` and `flaky`; `waived` is not a pass either and is shown as `[waived]` in `trace`, `matrix`, `safety-case` and `testplan`, never green, but raises no `W010`/`W615` because a human decision with a rationale is on record.
- **Measurements.** Each is `{name, value, limit?}`: `value` is a quantity in the expression language with its unit (`"4.2 ms"`), `limit` is `{op, value}` with `op` one of `<`, `<=`, `>`, `>=`, `==`. A limit is checked with the unit and dimension rules of the expression language. A record whose verdict is `pass` while a measurement violates its own limit, or whose limit and value differ in dimension, is a hard parse error. The tool never rewrites a verdict.
- **Provenance.** `build` (`commit`, `version`, either optional), `environment` (a `TestEnvironment` id or qualified name, resolved at ingest; REQ-TRS-RESENV-001), `operator`, `startedAt` and `finishedAt` (RFC 3339), and `artefacts`: a list of `{uri, hash}` where `hash` is `sha256:<hex>` or `blake3:<hex>`. Artefacts are **referenced, never copied** into the model or the sidecar. All provenance is optional; a record without it is valid.
- **One file is one run.** If a file holds several records with the same `(testCase, scenario, environment)`, the last one is kept (a retry or a correction, never two results) and the ingest says how many were superseded. A leading UTF-8 byte-order mark is ignored.
- **Fail closed.** As for `session-log`, a malformed record (empty `testCase`, unknown verdict, `waived` without `rationale`, malformed hash or timestamp, inconsistent measurement) is a hard parse error naming the record; nothing is written and an existing sidecar is untouched.
- **Storage.** The sidecar gains a `records` section holding the normalised records. The existing `by_leaf` and `by_scenario` maps are derived from it as a compatibility layer, so `trace`, `matrix`, `safety-case`, `testplan`, `W010` and `W615` read exactly what they read today. The new section is additive; a sidecar without it reads unchanged. `--run` and `--config` behave as for other formats (REQ-TRS-RUNHIST-001, REQ-TRS-CFGRES-001).
- **Merge.** A `records` ingest supersedes the records with the same `(testCase, scenario, environment)` key and keeps the rest, so a hardware-in-the-loop ingest does not erase bench results. `--replace` drops the records the ingest does not contain, restoring whole-section replacement.
- **Sanitising.** Free text from a record (names, rationale, operator) is stored and printed with control characters removed, as `one_line` already does for other formats. A secret in an artefact URI is the author's responsibility; the tool never fetches a URI.

## Documentation and spec (part of the acceptance of this requirement)

`prompts/help/ingest-results.md` and `prompts/help/results.md` (format, verdicts, `--replace`); the `ingest-results` section of `docs/cli/index.md` and the command row in `prompts/create-model.md`; the results chapter of `spec/markdown-sysml-format.md` (record schema, verdict roll-up, sidecar `records`) and, for any new code, **both** `prompts/spec/validation.md` and `docs/validation/rules.md`; a worked example `examples/results-records/` with a sample file; and the drift tests (`catalogue_tests`, `validation_docs_catalogue`, the help-example test) extended to cover them.
