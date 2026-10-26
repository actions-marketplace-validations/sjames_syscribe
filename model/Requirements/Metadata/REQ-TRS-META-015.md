---
type: Requirement
id: REQ-TRS-META-015
name: "Tagged values may be variant parameters"
status: draft
reqDomain: software
reqClass: system
tags:
  - metadata
---

A tagged value shall accept a feature-parameter placeholder substituted under `--config` (GH #375, #311).

## Behavior

- A value of the form `{{Features::GridProfile.nominalFrequencyHz}}` is a placeholder resolved against the selected `Configuration`, in the same way as for machines.
- Under `--config` the substituted value is type-, unit- and range-checked as an ordinary value; without `--config` the placeholder is left as written and is not type-checked.
- A placeholder naming an unknown or unselected parameter is reported like the machine placeholder (`E203` family).
