---
type: PlanningItem
id: PI-META-377
name: "Specialisation, profiles and a standard stereotype library (GH #377)"
status: done
itemType: feature
achieves:
  - REQ-TRS-META-030
  - REQ-TRS-META-031
  - REQ-TRS-META-032
  - REQ-TRS-META-033
evidence:
  - ref: TC-TRS-META-030
  - ref: TC-TRS-META-031
  - ref: TC-TRS-META-032
  - ref: TC-TRS-META-033
  - path: repo:crates/syscribe-model/tests/metadata_profiles.rs
  - path: repo:crates/syscribe/tests/metadata_profiles_cli.rs
  - path: repo:crates/syscribe/tests/metadata_profile_enable.rs
tags:
  - metadata
---

`supertype:` on a `MetadataDef` (030); stereotype profiles enabled by `[metadata] profiles` (031) — constraints and auto-apply gated, `W316`; the built-in `StdMetadata` library (032), added when referenced or `[metadata] stdlib = true`, read-only for `set`; `autoApply:` rules deriving applications (033). Limits: derived applications are not seen by diagrams or by the typing/constraint checks, and MCP/web write routes reject a built-in element only because its file does not exist.
