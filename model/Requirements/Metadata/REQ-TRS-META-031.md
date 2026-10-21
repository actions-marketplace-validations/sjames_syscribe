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

- `profile: true` (with optional `version:`) on a package `_index.md` declares a profile. `[metadata] profiles = ["Pkg", …]` in `.syscribe.toml` enables it. (`[profiles.<name>]` is already the *severity* profile table used by `--profile`; it is left alone.)
- An enabled profile activates its definitions' `constraints:` (`E327`) and its automatic applications (REQ-TRS-META-033); a disabled profile leaves the model unchanged. A definition outside any profile package is always checked. Type, range and enumeration checks (`E322`–`E326`) are not gated by a profile.
- `profile:` and `version:` are recognised on a package; elsewhere they are `W047`.
- An entry that names no package, or a package that is not `profile: true`, is `W316`.
- Definitions remain usable by reference whether or not the profile is enabled.
