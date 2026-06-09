# extref — find elements by external reference

## SYNOPSIS
    syscribe -m <root> extref <ref> [--json]

## DESCRIPTION
Finds every element whose `extRef:` (a link to an artifact in another tool — a
DOORS Next requirement, a SysML-tool element, a ticket) equals <ref> exactly.
Returns all matches; exits non-zero when nothing matches. A duplicate extRef
across elements warns W028.

## OPTIONS
    --json   Emit the matches as a JSON array.

## EXAMPLES
    syscribe -m model/ extref "DNG:4521"
    syscribe -m model/ extref "cameo://model/Engine#id-99" --json

## EXIT CODES
    0  at least one match    1  no element declares <ref>

## SEE ALSO
    find, show
