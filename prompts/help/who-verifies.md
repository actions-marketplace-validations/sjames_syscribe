# who-verifies — which test cases cover a requirement

## SYNOPSIS
    syscribe -m <root> who-verifies <req-id>

## DESCRIPTION
Lists the TestCases that verify a requirement, with their test level, Gherkin
scenario count, and status.

## EXAMPLES
    syscribe -m model/ who-verifies REQ-UAV-SAFE-001

## SEE ALSO
    trace, verification-depth, why
