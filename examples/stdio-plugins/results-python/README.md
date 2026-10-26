# Results plugins: a rig log and TAP

A results plugin converts a test report in any format into normalised result records
(`REQ-TRS-RESPLUG-001`, `docs/model-guide/stdio-plugins.md` section 10). It is a plain program that reads
one JSON request on stdin and writes one JSON object on stdout. Anything it returns is validated exactly like
`ingest-results --format records`, so a plugin cannot store anything the built-in format could not.

Both converters are standard-library Python, with no build step.

```bash
cd examples/stdio-plugins/results-python

# See exactly what a plugin returns, without merging or validating anything
syscribe -m model plugins run rig --report rig.log --dry-run

# Ingest the made-up rig log (--environment fills the rig the log does not name)
syscribe -m model ingest-results --format plugin:rig --environment TE-HIL-001 rig.log
syscribe -m model trace REQ-BRK-001

# TAP: the first word of a description is the key; `extref`-style keys join by the TestCase's extRef
syscribe -m model ingest-results --format plugin:tap --environment TE-HIL-001 run.tap
syscribe -m model results failures
```

What to notice:

* `rig.log` names the test by `RIG:BRK-1`, the `extRef` of `TC-BRK-001`; the record is stored under the
  TestCase id. `RIG:UNKNOWN-9` names nothing in the model, so it is reported and kept apart, and counts as
  evidence for nothing (`--strict` would reject the file).
* The log's measurement (`3.1ms` against `limit<5ms`) arrives as a unit-checked measurement.
* `--environment` and `--build-commit` fill provenance a record leaves empty and never override what a record
  says, so the log's own build (`abc1234`) wins over `--build-commit`.
* In `.syscribe.toml` the plugins are marked `kind = "results"`; without it, `--format plugin:rig` is refused
  (`E552`) because the entry would be a foreign-format plugin.
