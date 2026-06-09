# ls — list namespace children

## SYNOPSIS
    syscribe -m <root> ls [qname]

## DESCRIPTION
Lists the direct children of a namespace (default: the model root). For the
recursive form, use `tree`.

## EXAMPLES
    syscribe -m model/ ls
    syscribe -m model/ ls System::Software

## SEE ALSO
    tree, show
