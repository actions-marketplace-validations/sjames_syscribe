---
type: PartDef
name: ExecutionRuntime
supertype: Parts::Part
domain: software
satisfies:
  - REQ-TRS-ACTSEND-001
implementedBy:
  - repo:crates/syscribe-model/src/sm_run.rs
  - repo:crates/syscribe-model/src/act_run.rs
---

The deterministic interpreters behind `simulate`: the state-machine run (SCXML run-to-completion over the IR), the action-flow token run, and the composition of several parts on one virtual clock. A state machine that runs an executable activity emits the activity's `SendAction`s as its own messages.
