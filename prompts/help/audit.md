# audit — safety-readiness dashboard

## SYNOPSIS
    syscribe -m <root> audit [--json] [--profile <name>]

## DESCRIPTION
Rolls up a top-level readiness picture: requirement status split (overall and
per top-level package), SIL/ASIL distribution, per-configuration coverage %,
orphans (requirements with no test / no satisfying element, dangling TestCases,
no-trace requirements), and a single PASS/FAIL verdict.

## OPTIONS
    --profile <name>  Use a .syscribe.toml [profiles.<name>] policy as the bar.
    --json            Emit the whole rollup as one JSON document.

## DESCRIPTION (policy)
The verdict FAILS when any Error finding exists, any W306 (unsatisfied safety
mechanism) is present, or — with --profile — any finding the profile promotes.

## EXAMPLES
    syscribe -m model/ audit
    syscribe -m model/ audit --profile safety
    syscribe -m model/ audit --json

## EXIT CODES
    0  PASS    2  FAIL (verdict)    1  undefined --profile

## SEE ALSO
    validate, matrix, verification-depth, metrics
