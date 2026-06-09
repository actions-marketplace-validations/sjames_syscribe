# feature — one feature's card

## SYNOPSIS
    syscribe -m <root> feature <qname|name> [--json]

## DESCRIPTION
Shows a single FeatureDef's card: its doc, groupKind, requires/excludes,
parameters (with binding times), the Configurations that select it, and every
element it gates (whose appliesWhen names it). Errors on a non-FeatureDef arg.

## EXAMPLES
    syscribe -m model/ feature Features::Payload::Delivery

## SEE ALSO
    features, list --feature, why-active
