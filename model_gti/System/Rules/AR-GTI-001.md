---
type: ArchitectureRule
id: AR-GTI-001
name: The hardware trip path does not depend on software
kind: forbid-dependency
from: {tag: protection-hw}
to: {domain: software}
severity: error
---

REQ-GTI-HW-008 and ADR-GTI-SAFE-001: the hardware trip must disconnect the inverter even if the firmware is blind or faulty, so no part on the trip path (grid-voltage sensing, the control board's comparators and trip drivers, the power stage and the AC output relay) may take an input from, or be built from, a software component. `validate` reports the first software component a trip-path part depends on, with the path.
