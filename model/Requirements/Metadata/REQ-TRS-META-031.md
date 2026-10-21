---
type: Requirement
id: REQ-TRS-META-031
name: "Stereotypes ship as profiles enabled in configuration"
status: draft
reqDomain: software
reqClass: system
tags:
  - metadata
---

A package marked `profile: true` shall group `MetadataDef`s as a reusable, versioned profile enabled by `[profiles]` (GH #377).

## Behavior

- `profile: true` (with optional `version:`) on a package `_index.md` declares a profile. `[profiles.<name>]` in `.syscribe.toml` enables it, as for MagicGrid.
- An enabled profile activates its definitions' `checks:` and automatic applications; a disabled profile leaves the model unchanged.
- Definitions remain usable by reference whether or not the profile is enabled.
