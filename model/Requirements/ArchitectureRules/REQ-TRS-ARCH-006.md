---
type: Requirement
id: REQ-TRS-ARCH-006
name: "allow-only and layering rules"
status: draft
reqDomain: software
reqClass: system
tags:
  - architecture-rules
---

An `ArchitectureRule` shall be able to say what a set of components may depend on, and to order them in layers (GH #369).

## Behavior

- `kind: allow-only`: a component chosen by `from` may depend directly only on components chosen by `from` or `to` (itself included). A direct dependency on any other component is `E660` (`W660` by severity), naming the rule, the dependent and the dependency.
- `layer:` on a `PartDef`/`Part` names its layer; `layer` is also a selector key. `kind: layering` with `layers: [top, ..., bottom]` forbids a direct dependency from a component in a layer on one in a layer above it (itself and any layer below, skipping included, are allowed). Components with no layer, or a layer not listed, are unconstrained. Same code.
- `layering` needs `layers` (a non-empty list of distinct names), and neither `layering` nor `max-depth` needs `from`/`to`; `allow-only` needs both; a malformed rule is `E662`.
- `layer` on any other element type is `W047`.
