---
type: Requirement
id: REQ-TRS-SMCHECK-001
name: "A state machine can be model-checked exhaustively over a declared, finite environment, with a replayable counterexample for every violation"
status: draft
reqDomain: software
reqClass: system
tags:
  - execution
  - state-machine
  - verification
  - model-checking
---

`syscribe-model` shall explore the reachable behaviour of an executable `StateDef` (REQ-TRS-SMEXEC-001) exhaustively, over an environment the author declares finite, and shall report for each property one of **proved**, **violated** (with a counterexample) or **inconclusive**. Simulation shows one path; this answers "can it ever happen?". The interpreter stays the single definition of the machine's behaviour: the checker drives it and never re-implements it, so a result cannot disagree with `simulate`.

## Behavior

- **Engine.** Explicit-state, breadth-first search driving the existing interpreter (REQ-TRS-SMEXEC-001), so the first violation found is the shortest. This needs the run to be snapshot-able and restorable; a state is keyed by the active-state set, the variable values (quantised as in the interpreter), the armed timers as *offsets from now*, the change-observation state and the pending internal queue, with the absolute clock excluded, so equal futures are one state. A SAT or bounded encoding is a possible later engine and needs its own ADR before it is started; it is not part of this requirement.
- **Environment, declared and finite.** The search chooses only what the environment controls: which external event arrives, with which payload, which input variable is set (`set`), and how far the clock advances. Each is drawn from a declared set: the events of the machine, a payload field or input variable with an enumerated or integer-range domain, and a set of clock advances. An input with no declared finite domain is an error, never guessed. Declarations live in the property's `TestCase`, in the Gherkin vocabulary of REQ-TRS-SMSCEN-001 (new steps; a step outside the vocabulary is `E886` as today), so a property is traced with `verifies:` and recorded as evidence like any scenario.
- **Time.** The clock advances to the next due timer or by a declared amount, never by an unbounded real. Inputs therefore arrive at the current instant, at timer boundaries, or at declared offsets. This is an under-approximation of "any arrival time" and every result states it; a timed-automata style abstraction is out of scope.
- **Built-in checks, no property needed.** (a) a state no explored run enters (dynamic dead state, the runtime counterpart of `W070`); (b) a reachable trap or deadlock: a non-final configuration that no declared environment input can leave and no timer will; (c) a reachable run error (expression error, type or dimension error, microstep-bound livelock) with its location; (d) every strict-mode finding of REQ-TRS-SMEXEC-001 (equal-priority tie, cross-region exit conflict, two regions assigning one variable) on **any** reachable state, not only on one script.
- **User properties.** In the TestCase: `the machine never reaches <State>`, and `<expression> always holds` (an invariant over variables, evaluated in every stable configuration). Liveness properties ("eventually") are not in this phase. Expressions are the typed, unit-checked expression language, checked statically as for guards (`E935`).
- **Verdicts are honest.** *Proved* only when the whole space was exhausted within the declared environment and no bound cut the search. Reaching `--max-states` or `--max-depth` yields *inconclusive*, with the count explored and the bound that stopped it, and **never** *proved*. A violation is *violated*.
- **Counterexample.** The shortest path as a `simulate --do` script plus its trace. Before it is reported it is replayed through the unmodified interpreter and must reproduce the violation; a counterexample that does not replay is an internal error, not a result.
- **Determinism and scope of claim.** The result is a pure function of the machine, the declarations and the bounds; the order of exploration is fixed. What is verified is the behaviour under the interpreter's semantics, including its lock-step treatment of parallel regions (regions are not interleaved; scheduler races are reported only through the strict-mode findings of (d), not enumerated). Plants and FMUs are continuous and unsupported: a machine composed with one is reported as outside the checkable subset, not approximated.
- **Command.** `syscribe model-check <StateDef|TestCase> [--max-states N] [--max-depth N] [--json] [--record [--run <id>] [--config <C>]]`, with the `--config` placeholder resolution of `simulate`. Exit `0` only when every property is *proved*; `1` when any is *violated*; `2` on an error in the model or the declarations; a non-zero exit for *inconclusive* (fail closed) unless explicitly allowed. MCP exposes a read-only `model_check`.

## Phases

1. **A single flat or hierarchical machine**: the engine, the finite environment, the built-in checks (a)–(d), `never reaches` and invariants, counterexample replay, the command, evidence recording.
2. **Composed systems** (REQ-TRS-SYSCOMP-001): the product of parts with message queues and connection delays; queue bounds are declared like any other input domain.
3. **Action flows** (REQ-TRS-ACTEXEC-001): token state as part of the key, for deadlock and unreachable-node checks.
4. **Liveness and fairness** (`eventually`, "always eventually leaves"), which need cycle detection and a fairness assumption; deferred until phase 1 has users.

## Not covered

Continuous dynamics and plants; real-valued or unbounded input domains; arbitrary interleavings of parallel regions; unbounded timing analysis, schedulability and worst-case latency (a separate requirement); probabilistic behaviour.

## Documentation and spec (part of the acceptance of this requirement)

A `model-check` help page and its row in `docs/cli/index.md` and `prompts/create-model.md`; a model-guide chapter (declaring the finite environment, reading a verdict and a counterexample, why *inconclusive* is not *proved*); the Gherkin vocabulary table of the executable-evidence guide and the spec (`never reaches`, `always holds`, the environment declarations, and `E886` for a step outside it); both validation catalogues for any new code; the `TestCase` skeleton printed by `syscribe template TestCase` when a property test case is scaffolded; MCP documentation for `model_check`; a worked example `examples/model-check/`; and the drift tests.
