# (default) — full validation report

## SYNOPSIS
    syscribe -m <root>

## DESCRIPTION
With no subcommand, syscribe prints the full 10-section Markdown report: element
inventory, requirement coverage matrix, traceability summary, safety/security
rollups, and the findings tables. For findings only, use `validate`; for a
readiness verdict, use `audit`.

## EXAMPLES
    syscribe -m model/
    syscribe -m model/ > report.md

## SEE ALSO
    validate, audit
