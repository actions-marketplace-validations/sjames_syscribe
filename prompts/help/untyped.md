# untyped — elements with no type: field

## SYNOPSIS
    syscribe -m <root> untyped

## DESCRIPTION
Lists files that parsed but carry no `type:` field (W008). These are ignored by
most commands; this surfaces them so they can be typed or removed.

## EXAMPLES
    syscribe -m model/ untyped

## SEE ALSO
    types, validate
