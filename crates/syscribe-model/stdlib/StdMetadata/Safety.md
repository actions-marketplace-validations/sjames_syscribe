---
type: MetadataDef
name: Safety
features:
  - name: level
    typedBy: ScalarValues::Integer
    range: "0..4"
  - name: goal
    typedBy: ScalarValues::String
---

Safety relevance: an integrity `level` (0 = none … 4 = highest) and the safety `goal` it serves.
