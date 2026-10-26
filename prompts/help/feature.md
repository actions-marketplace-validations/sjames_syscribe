# feature — one feature's card

## SYNOPSIS
    syscribe -m <root> feature <qname|name|FEAT-id> [--json]

## DESCRIPTION
Shows a single FeatureDef's card: its doc, groupKind, requires/excludes,
parameters (with binding times), the Configurations that select it, and every
element it gates (whose appliesWhen names it). The argument may be the qualified name, the bare name or the stable `FEAT-*` id.
Errors (exit 1) on a non-FeatureDef arg.

## EXAMPLES
    syscribe -m model/ feature Features::Payload::Delivery

## SEE ALSO
    features, list --feature, why-active
