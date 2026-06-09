# features — feature-model overview

## SYNOPSIS
    syscribe -m <root> features [--json]

## DESCRIPTION
Prints the feature model as a tree: each FeatureDef's groupKind, requires/
excludes, parameters, and a "selected in N/M configurations" rollup.

## EXAMPLES
    syscribe -m model/ features

## SEE ALSO
    feature, feature-check, matrix --features
