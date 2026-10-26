# Robot Framework results

`ingest-results --format robot` reads a Robot Framework `output.xml` (Robot 6 and 7) and joins its tests to
`TestCase`s by **tag**. See `syscribe help ingest-results` and `docs/guides/tests.md`.

```bash
cd examples/results-robot
robot --output output.xml --log NONE --report NONE suite.robot     # or use the sample output.xml
syscribe -m model ingest-results output.xml --environment TE-HIL-001 --build-commit abc1234 --operator alice
syscribe -m model trace REQ-BRK-001
syscribe -m model results failures
```

`ingest-results` recognises a Robot file by its `<robot>` root, so `--format robot` is optional for `.xml`.

How the tags in `suite.robot` map:

| Robot test | Tags | Becomes |
|---|---|---|
| Request reaches the actuator in time | `TC-BRK-001` | the scenario of that name in `TC-BRK-001` (pass) |
| Request during a bus fault is not lost | `TC-BRK-001` | the scenario of that name (fail, with Robot's message) |
| Parking brake holds | `extref:TAP:parking-brake` | `TC-BRK-003`, found through its `extRef` (pass) |
| A skipped one | `TC-BRK-002` | `TC-BRK-002` (not-run: Robot's SKIP) |
| Untagged test | none | reported as unmatched, evidence for nothing (`--strict` rejects the file) |
| Not a testcase tag | `REQ-BRK-001`, `smoke` | the same: only a tag that is a TestCase counts |

What to notice:

* A test named exactly like a `Scenario:` title is recorded against that scenario; any other test is the
  verdict of the whole TestCase. Several tests for one TestCase count as one: the worst verdict decides and
  the first failing test leads the detail.
* A test that never ran because a parent suite's setup failed is `blocked`, not `fail`: Robot reports it as
  FAIL, but nothing about the product was tested.
* Robot writes local time with no zone. The times are recorded as written with the zone you give:
  `--utc-offset +02:00` (default `Z`).
* `--artefact log.html` references Robot's log by its hash; the file is never copied.
