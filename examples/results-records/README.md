# Results records: system tests on two rigs

`ingest-results --format records` reads results that are not JUnit: system, integration and
hardware-in-the-loop runs, with measurements and provenance. See `syscribe help ingest-results`.

```bash
cd examples/results-records
syscribe -m model ingest-results --format records bench.json   # both scenarios pass on the bench
syscribe -m model trace REQ-BRK-001                            # TC-BRK-001 [pass]
syscribe -m model ingest-results --format records hil.json     # the HIL rig fails one and waives one
syscribe -m model trace REQ-BRK-001                            # TC-BRK-001 [fail]: the worst result wins
```

```bash
syscribe -m model testplan TP-BRK-001                          # Verdict: fail (the HIL rig fails)
syscribe -m model validate | grep W615                         # TC-BRK-001 @ TE-HIL-001 (FAILED)
```

What to notice:

* `TP-BRK-001` runs on both rigs, so it is judged on each: the bench pass does not rescue the HIL failure.
  A plan that listed only the bench would pass. `--environment TE-HIL-001` names the rig for a report that
  does not say.

* The bench results are **kept** when the HIL file is ingested (records are superseded only per test,
  scenario and environment). `--replace` would drop them.
* The failing scenario carries its measurement (`6.4 ms` against a limit of `< 5 ms`); a `pass` with that
  measurement would be rejected.
* The `waived` scenario needs its rationale and is never counted as a pass. With the failure fixed it would
  show as `[waived]`.
* The log is referenced by URI and hash; it is never copied into the model.
