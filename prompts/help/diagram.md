# diagram — diagram authoring workflow

## SYNOPSIS
    syscribe -m <root> diagram <list|measure|compose> [args]

## DESCRIPTION
Drives the companion-SVG diagram workflow: `diagram list` enumerates diagrams,
`diagram measure` emits element sizes, and `diagram compose --output <path>`
renders a *.layout.json into an SVG. (Layout files are ephemeral inputs and must
not be committed — name them *.layout.json so .gitignore excludes them.)

## EXAMPLES
    syscribe -m model/ diagram list
    syscribe -m model/ diagram measure UAV::SystemBDD
    syscribe -m model/ diagram compose UAV::SystemBDD --output model/Diagrams/SystemBDD.svg

## SEE ALSO
    render
