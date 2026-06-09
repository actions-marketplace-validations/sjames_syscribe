# configure — assisted configuration

## SYNOPSIS
    syscribe -m <root> configure <Configuration> [--json]

## DESCRIPTION
From a partial selection in a Configuration, reports satisfiability and which
features are forced vs free. Exits non-zero if the partial selection is
contradictory.

## EXAMPLES
    syscribe -m model/ configure CONF-UAV-DELIVERY-001

## SEE ALSO
    feature-check --deep, features, validate --config
