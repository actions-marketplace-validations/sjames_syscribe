---
type: Requirement
id: REQ-TRS-META-020
name: "Filter by tagged value with --where meta"
status: draft
reqDomain: software
reqClass: system
tags:
  - metadata
---

`list`, `find`, `ls` and `digest` shall filter on stereotype tagged values (GH #376).

## Behavior

- `--where meta.<Def>.<key><op><value>` accepts `=`, `!=`, `=~`, `~=`, the ordering operators `<` `<=` `>` `>=`, a bare `meta.<Def>.<key>` for presence of a value, and a bare `meta.<Def>` for presence of the stereotype. An ordering operator needs a number; anything else is an error naming the supported forms. `!=` does not match an element lacking the value. `<Def>` matches by qualified name or last segment, as `--metadata` does.
- Defaults (REQ-TRS-META-012) and inherited values (REQ-TRS-META-030) take part.
- `list PartDef --where meta.SafetyCritical.integrity>=2` returns exactly the parts that apply that stereotype with that value; an element without the stereotype does not match and is not an error.
- The MCP equivalents accept the same expression.
