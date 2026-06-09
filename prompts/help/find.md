# find — fuzzy search by name, ID, or content

## SYNOPSIS
    syscribe -m <root> find <pattern>

## DESCRIPTION
Searches element names, stable IDs, and documentation bodies, ranking results by
relevance. Use `extref` to look up by external reference, `list` to enumerate by
type.

## EXAMPLES
    syscribe -m model/ find throttle
    syscribe -m model/ find "brake release"

## SEE ALSO
    list, extref, show
