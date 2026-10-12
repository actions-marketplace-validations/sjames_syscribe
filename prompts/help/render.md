# render — print a Diagram element's embedded diagram with element links injected

## SYNOPSIS
    syscribe -m <root> render <diagram-path>

## DESCRIPTION
Prints the diagram embedded in a Diagram element's body to stdout, with links to
the model elements it shows injected. <diagram-path> is the element's file path
(or a path suffix of it), not a qualified name.

- `diagramKind: Mermaid` — the ```mermaid block, with a `click <node> href "<path>" _blank`
  directive added for every `%% link: <NodeId> <qname-or-id>` comment line in the block.
  Nodes are not detected from their labels: a block with no `%% link:` lines gets no
  `click` lines (an unresolved target is warning `W410`).
- any other `diagramKind` — the embedded ```svg block, with `<a href>` wrappers
  added around shapes whose entry in the frontmatter `shapes:` list carries a `link:`.

Mermaid example, inside the block:

    graph TD
      PERF["Mission Performance"]
      %% link: PERF REQ-UAV-PERF-000

It does not lay out or generate a diagram: companion SVG files come from the
browser (Save companion SVG) or from `plantuml render`.

## EXIT CODES
    0  printed    1  no element at the path, no `diagramKind:`, or the expected
                     ```mermaid / ```svg block is missing

## EXAMPLES
    # prints the block unchanged unless it has `%% link:` lines (the bundled one has none)
    syscribe -m model/ render model/Diagrams/RequirementTraceMermaid.md

## SEE ALSO
    plantuml
