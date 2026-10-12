# scaffold-gherkin — generate/align Gherkin scenarios for a TestCase

## SYNOPSIS
    syscribe -m <root> scaffold-gherkin <TC-id> [--fix]

## DESCRIPTION
Generates or aligns the Gherkin Scenario blocks of a TestCase from its
testFunctions: mappings (so each named function has a matching scenario). With
--fix, writes the alignment back to the file.

Without --fix it prints a preview of the whole Feature: every scenario appears with
`<precondition>`/`<action>`/`<expected outcome>` placeholders, marked `(exists)` or
`(missing)` in a trailing comment; only the missing ones are inserted by --fix, and
existing scenarios keep their real steps. A TestCase with no `testFunctions[].scenario`
entry prints "nothing to scaffold" and exits 1.

## OPTIONS
    --fix   Apply the changes to the TestCase file.

## EXAMPLES
    # needs a TestCase with testFunctions[].scenario entries (none in the bundled models)
    syscribe -m <root> scaffold-gherkin TC-SCHED-BITMAP-001
    syscribe -m <root> scaffold-gherkin TC-SCHED-BITMAP-001 --fix

## SEE ALSO
    who-verifies, validate
