# trace — full traceability slice for a requirement

## SYNOPSIS
    syscribe -m <root> trace <qname|req-id> [--linked-only]

## DESCRIPTION
Shows a requirement's complete traceability slice: parents (derivedFrom), the
breakdown ADR, the SafetyGoal it derives from, the architecture that satisfies
it, and the TestCases that verify it. When a results sidecar is present, each
verifying TestCase is annotated with its ingested verdict ([pass]/[fail]/[unknown]).

## OPTIONS
    --linked-only   Ignore ingested results; show linked tests without verdicts.

## EXAMPLES
    syscribe -m model/ trace REQ-UAV-NAV-001
    syscribe -m model/ trace REQ-UAV-NAV-001 --linked-only

## SEE ALSO
    why, who-verifies, links, verification-depth
