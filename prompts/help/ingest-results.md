# ingest-results — ingest external test results (enables W010)

## SYNOPSIS
    syscribe -m <root> ingest-results [--format cargo-json|junit] <file>

## DESCRIPTION
Parses an external test report (libtest JSON or JUnit XML), reduces it to a
per-test pass/fail/ignore verdict keyed by leaf name, and writes the sidecar at
<root>/.syscribe/results.json. Once present, `validate` emits W010 for active
TestCases whose functions last failed/were skipped/were missing, and
`matrix`/`trace` show executed-evidence verdicts.

## OPTIONS
    --format <fmt>   cargo-json (libtest JSON) or junit (JUnit XML); inferred from extension if omitted.

## EXAMPLES
    cargo test -- -Z unstable-options --format json | syscribe -m model/ ingest-results --format cargo-json /dev/stdin
    syscribe -m model/ ingest-results results.xml

## SEE ALSO
    validate (W010), matrix, trace
