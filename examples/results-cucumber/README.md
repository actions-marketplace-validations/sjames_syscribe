# Cucumber results

`ingest-results --format cucumber` reads the report of Cucumber's `json` formatter (Cucumber-JVM, Cucumber.js and
Cucumber-Ruby) and joins its scenarios to `TestCase`s by **tag**. See `syscribe help ingest-results`.

```bash
cd examples/results-cucumber
# produce a report (Cucumber.js shown; features/brake.feature is the suite)
npm install @cucumber/cucumber        # then write the step definitions
npx cucumber-js --format json:cucumber-js.json
# or use the samples here
syscribe -m model ingest-results cucumber-js.json --environment TE-HIL-001 --build-commit abc1234
syscribe -m model trace REQ-BRK-001
syscribe -m model results failures
```

`ingest-results` recognises the report by its shape, so `--format cucumber` is optional.

The three reports here are the same suite run three ways:

| File | Produced by | Notes |
|---|---|---|
| `cucumber-js.json` | Cucumber.js 11.3.0 (real output) | background steps and hooks are steps of each scenario; outline rows share an id and differ by `line`; a retried scenario appears once, as its last attempt |
| `cucumber-ruby.json` | Cucumber-Ruby 3.1.2 (real output) | a separate `background` element before each scenario; hooks in `before`/`after` arrays (a failing `Before` is on the background); outline rows have ids `;;2`, `;;3`, … |
| `cucumber-jvm.json` | **written by hand** in the shape the JVM formatter documents | start timestamps, nanosecond durations, embeddings, and one element per retry attempt. Cucumber-JVM could not be run to produce it |

How scenarios become records:

| Scenario | Becomes |
|---|---|
| tagged `@TC-BRK-001`, titled like a `Scenario:` of it | that scenario's record; `pass`, or `fail` with the first failing step and its message |
| tagged `@extref:TAP:parking-brake` (any case) | the TestCase with that `extRef` |
| a row of `Scenario Outline: Latency by speed` | `Latency by speed [row N]`; the outline's own title carries the worst row |
| a step `pending`, `undefined` or `skipped`, or no steps of its own | `not-run` |
| a step `failed` or `ambiguous`, or a failing hook | `fail` |
| the same scenario twice, failing then passing | `flaky`; failing on the last attempt is `fail` |
| no tag that names a TestCase | reported as unmatched, evidence for nothing (`--strict` rejects the file) |

Tags on the feature apply to every scenario in it. Embedded screenshots and step output are never stored.
