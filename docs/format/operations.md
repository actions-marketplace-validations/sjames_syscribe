# Operations

`FORMAT · OPERATIONS`

The `operations:` field declares callable operations (synchronous) or receptions (asynchronous) on a definition element. It maps directly to SysMLv2 `OperationDef` (synchronous) and `ReceptionDef` (asynchronous) owned by the enclosing definition.

## Where it is valid

`operations:` is valid on any definition element. The primary use cases are:

- `PortDef` — operations exposed through this port
- `InterfaceDef` — operations spanning both ends of the interface
- `ConnectionDef` — operations on the connection itself
- `PartDef` — operations on the structural component

## Operation entry schema

| Field | YAML type | Required | Default | Description |
|---|---|---|---|---|
| `name` | string | **Required** | — | Operation name |
| `doc` | string | optional | absent | Inline documentation for this operation |
| `isQuery` | bool | optional | `false` | Side-effect free — read-only operation |
| `isAsync` | bool | optional | `false` | Reception — caller does not block; `returnType` must be absent |
| `parameters` | list | optional | `[]` | Typed, directional parameters |
| `returnType` | string | optional | absent | Qualified name of the return type; mutually exclusive with `isAsync: true` |
| `errors` | string or list | optional | absent | Error types the operation can raise (`ItemDef`, `AttributeDef` or an enumeration); not on an async operation |
| `realizedBy` | string | optional | absent | The `ActionDef` that implements the operation |

## Parameter entry schema

| Field | YAML type | Required | Default | Description |
|---|---|---|---|---|
| `name` | string | **Required** | — | Parameter name |
| `typedBy` | string | optional | absent | Qualified name of the type; unresolved types fire **W404** |
| `direction` | string | optional | `in` | `in`, `out`, or `inout` |
| `multiplicity` | string | optional | `"1"` | Cardinality |
| `unit` | string | optional | absent | Unit for scalar quantities |

## Rules

- `isAsync: true` and `returnType:` are mutually exclusive. An async reception cannot return a value.
- `typedBy` on a parameter and `returnType` at the operation level are checked against the model resolver. Unresolved types fire **W404** (warning, not error — standard library types such as `ScalarValues::Boolean` and `ScalarValues::Integer` are common unregistered external namespaces).

## Example — PortDef with sync query and async reception

```yaml
---
type: PortDef
name: TelemetryPortDef
supertype: Ports::Port
features:
  - name: packet
    typedBy: Items::TelemetryPacket
    direction: out
    multiplicity: "0..*"
operations:
  - name: requestLatest
    doc: "Return the most recent telemetry packet synchronously."
    isQuery: true
    parameters: []
    returnType: Items::TelemetryPacket
  - name: subscribe
    doc: "Register for periodic telemetry push at the given interval."
    isAsync: true
    parameters:
      - name: intervalMs
        typedBy: ScalarValues::Integer
        direction: in
---
Port definition for outgoing telemetry data.
```

## Example — InterfaceDef with operations spanning both ends

```yaml
---
type: InterfaceDef
name: CommandInterface
ends:
  - name: commander
    typedBy: Interfaces::ControlPortDef
  - name: executor
    typedBy: Interfaces::ControlPortDef
    isConjugated: true
operations:
  - name: executeCommand
    parameters:
      - name: cmd
        typedBy: Items::ControlCommand
        direction: in
    returnType: ScalarValues::Boolean
  - name: abort
    isAsync: true
    parameters: []
---
Interface definition for command dispatch between a commander and an executor.
```

## Realisation by an action

`realizedBy: <ActionDef>` ties the contract to the behaviour. The action's parameters must match the operation's:

- every operation parameter has an action parameter of the same `name` and `direction` (default `in`), with the same `typedBy` and the same `unit` when both give one;
- the action has no parameter the operation lacks, apart from its one `direction: return` parameter, which is the operation's `returnType` (both or neither, the same type when both give one); an `isAsync` operation has no return.

Each difference is one `E672` on the element that declares the operation. Operations that declare neither `errors` nor `realizedBy` are not looked at.

```yaml
operations:
  - name: compute
    parameters:
      - {name: x, typedBy: ScalarValues::Integer, direction: in}
    returnType: ScalarValues::Boolean
    errors: [Items::Fault]
    realizedBy: Behaviour::Compute     # an ActionDef with parameters x (in) and one return
```

## Calling an operation

A `SendAction` or `PerformAction` in an `ActionDef`'s `subActions:` (at any depth, including `then`/`else`/`body`) names the operation it calls with `operation: <owner>::<name>`; its `parameters:` entries (`name`, `value`) are the arguments.

```yaml
subActions:
  - name: sample
    kind: PerformAction
    operation: Sensors::Adc::read
    parameters:
      - {name: channel, value: "1"}
```

The owner must resolve and declare that operation, else `E673`. An argument that is not an `in`/`inout` parameter of the operation, or an `in`/`inout` parameter with no argument, is `E674`. The value of each argument is typed statically (a number with an optional unit such as `2 s`, a boolean, a string) and compared with its parameter's `typedBy` and `unit`: a different kind or unit dimension is `E675` (dimensions are compared, so `2 ms` fills a parameter in `s`). Names and parameters of other types are not checked. A sub-action without `operation:` is not looked at.

## Validation

| Code | Severity | Condition |
|---|---|---|
| W404 | Warning | Operation `typedBy` (parameter) or `returnType` does not resolve to a known element |
| E670 | Error | `errors:` entry does not resolve or is not a value type; `errors:` on an async operation |
| E671 | Error | `realizedBy:` does not resolve to an `ActionDef` |
| E672 | Error | The realising action's signature differs from the operation's (one per difference) |
| E673 | Error | A sub-action's `operation:` is on something other than a `SendAction`/`PerformAction`, or does not name an operation (`<owner>::<name>`: the owner does not resolve or declares no such operation) (REQ-TRS-OPCALL-001). |
| E674 | Error | A call that names an operation passes an argument that is not one of its input parameters, or omits an `in`/`inout` parameter (REQ-TRS-OPCALL-002). |
| E675 | Error | An argument value of a call that names an operation has a different kind (number, boolean, string) or unit dimension than its parameter declares, found statically; names and untypable values are not checked (REQ-TRS-OPCALL-003). |
