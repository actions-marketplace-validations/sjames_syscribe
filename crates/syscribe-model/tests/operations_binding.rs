//! TC-TRS-OPS-001..003 / REQ-TRS-OPS-001..003 (GH #364, items 1 and 2): `errors:` and `realizedBy:` on an
//! operation, and the signature match between an operation and the `ActionDef` that realises it.
//!
//! Written before the implementation.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use syscribe_model::element::RawElement;
use syscribe_model::validator::validate;
use syscribe_model::walker::walk_model;

fn write(root: &Path, rel: &str, content: &str) {
    let path = root.join(rel);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, content).unwrap();
}

fn tempdir() -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let p = std::env::temp_dir().join(format!("syscribe-ops-{}-{}", std::process::id(), N.fetch_add(1, Ordering::Relaxed)));
    std::fs::create_dir_all(&p).unwrap();
    p
}

/// `Ops::Svc` (an `InterfaceDef`) declares `operations` (YAML list items, already indented); `Ops::Act` (an
/// `ActionDef`) has `act_params`; `Ops::Fault` is an ItemDef, `Ops::Mode` an EnumerationDef, `Ops::Thing` a PartDef.
fn model(operations: &str, act_params: &str) -> Vec<RawElement> {
    let r = tempdir();
    write(&r, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(&r, "Ops/_index.md", "---\ntype: Package\nname: Ops\n---\n");
    write(&r, "Ops/Fault.md", "---\ntype: ItemDef\nname: Fault\n---\n\nF.\n");
    write(&r, "Ops/Mode.md", "---\ntype: EnumerationDef\nname: Mode\n---\n\nM.\n");
    write(&r, "Ops/Thing.md", "---\ntype: PartDef\nname: Thing\n---\n\nT.\n");
    write(&r, "Ops/Act.md", &format!("---\ntype: ActionDef\nname: Act\nparameters:\n{act_params}---\n\nA.\n"));
    write(&r, "Ops/Svc.md", &format!("---\ntype: InterfaceDef\nname: Svc\noperations:\n{operations}---\n\nS.\n"));
    walk_model(&r).unwrap()
}

fn codes(els: &[RawElement], code: &str) -> Vec<(String, String)> {
    validate(els).findings.iter().filter(|f| f.code == code).map(|f| (f.file.clone(), f.message.clone())).collect()
}

/// The matching pair: `compute(x: Integer in, y: Real out) : Boolean`.
const OP: &str = "  - name: compute\n    parameters:\n      - {name: x, typedBy: ScalarValues::Integer, direction: in, unit: m}\n      - {name: y, typedBy: ScalarValues::Real, direction: out}\n    returnType: ScalarValues::Boolean\n    realizedBy: Ops::Act\n";
const ACT: &str = "  - {name: x, typedBy: ScalarValues::Integer, direction: in, unit: m}\n  - {name: y, typedBy: ScalarValues::Real, direction: out}\n  - {name: result, typedBy: ScalarValues::Boolean, direction: return}\n";

// ── REQ-TRS-OPS-001: errors ────────────────────────────────────────────────

#[test]
fn errors_naming_value_types_are_quiet() {
    for errors in ["errors: Ops::Fault", "errors: [Ops::Fault, Ops::Mode]"] {
        let els = model(&format!("  - name: op\n    returnType: ScalarValues::Boolean\n    {errors}\n"), "  - {name: a, direction: in}\n");
        assert!(codes(&els, "E670").is_empty(), "{errors}: {:?}", validate(&els).findings);
    }
}

#[test]
fn an_unresolved_error_or_one_that_is_not_a_value_type_is_e670() {
    let missing = model("  - name: op\n    errors: [Ops::Nowhere]\n", "  - {name: a, direction: in}\n");
    assert_eq!(codes(&missing, "E670").len(), 1);
    assert!(codes(&missing, "E670")[0].1.contains("Ops::Nowhere"));
    let part = model("  - name: op\n    errors: [Ops::Thing]\n", "  - {name: a, direction: in}\n");
    assert_eq!(codes(&part, "E670").len(), 1, "a PartDef is not an error type");
}

#[test]
fn an_asynchronous_operation_cannot_declare_errors() {
    let els = model("  - name: op\n    isAsync: true\n    errors: [Ops::Fault]\n", "  - {name: a, direction: in}\n");
    let e = codes(&els, "E670");
    assert_eq!(e.len(), 1, "{e:?}");
    assert!(e[0].1.contains("op") && e[0].1.to_lowercase().contains("async"), "{e:?}");
}

#[test]
fn operations_on_any_element_type_are_checked() {
    let r = tempdir();
    write(&r, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(&r, "P.md", "---\ntype: PartDef\nname: P\noperations:\n  - name: op\n    errors: [Nowhere]\n---\n\nP.\n");
    let els = walk_model(&r).unwrap();
    assert_eq!(codes(&els, "E670").len(), 1);
}

// ── REQ-TRS-OPS-002: realizedBy ────────────────────────────────────────────

#[test]
fn realized_by_an_action_def_is_accepted() {
    let els = model(OP, ACT);
    assert!(codes(&els, "E671").is_empty(), "{:?}", validate(&els).findings);
}

#[test]
fn realized_by_something_else_is_e671() {
    for target in ["Ops::Nowhere", "Ops::Thing"] {
        let els = model(&OP.replace("Ops::Act", target), ACT);
        let e = codes(&els, "E671");
        assert_eq!(e.len(), 1, "{target}: {e:?}");
        assert!(e[0].1.contains("compute") && e[0].1.contains(target), "{e:?}");
        assert!(codes(&els, "E672").is_empty(), "a bad target has no signature to compare");
    }
}

// ── REQ-TRS-OPS-003: the signature match ───────────────────────────────────

#[test]
fn a_matching_signature_is_quiet() {
    let els = model(OP, ACT);
    assert!(codes(&els, "E672").is_empty(), "{:?}", validate(&els).findings);
}

fn mismatch(op: &str, act: &str) -> Vec<String> {
    codes(&model(op, act), "E672").into_iter().map(|(_, m)| m).collect()
}

#[test]
fn a_parameter_the_action_lacks_is_one_finding_naming_it() {
    let m = mismatch(OP, "  - {name: x, typedBy: ScalarValues::Integer, direction: in, unit: m}\n  - {name: result, typedBy: ScalarValues::Boolean, direction: return}\n");
    assert_eq!(m.len(), 1, "{m:?}");
    assert!(m[0].contains("compute") && m[0].contains("Ops::Act") && m[0].contains("'y'"), "{m:?}");
}

#[test]
fn an_extra_action_parameter_is_reported() {
    let m = mismatch(OP, &format!("{ACT}  - {{name: z, typedBy: ScalarValues::Integer, direction: in}}\n"));
    assert_eq!(m.len(), 1, "{m:?}");
    assert!(m[0].contains("'z'"), "{m:?}");
}

#[test]
fn a_different_direction_is_reported() {
    let m = mismatch(OP, &ACT.replace("name: y, typedBy: ScalarValues::Real, direction: out", "name: y, typedBy: ScalarValues::Real, direction: in"));
    assert_eq!(m.len(), 1, "{m:?}");
    assert!(m[0].contains("'y'") && m[0].contains("direction"), "{m:?}");
}

#[test]
fn a_different_type_is_reported_and_an_absent_type_is_not() {
    let m = mismatch(OP, &ACT.replace("name: x, typedBy: ScalarValues::Integer", "name: x, typedBy: ScalarValues::Real"));
    assert_eq!(m.len(), 1, "{m:?}");
    assert!(m[0].contains("'x'") && m[0].contains("type"), "{m:?}");
    // the action leaves a type out: nothing to compare
    assert!(mismatch(OP, &ACT.replace("typedBy: ScalarValues::Real, ", "")).is_empty());
}

#[test]
fn a_different_unit_is_reported_and_an_absent_unit_is_not() {
    let m = mismatch(OP, &ACT.replace("unit: m", "unit: s"));
    assert_eq!(m.len(), 1, "{m:?}");
    assert!(m[0].contains("'x'") && m[0].contains("unit"), "{m:?}");
    assert!(mismatch(OP, &ACT.replace(", unit: m", "")).is_empty(), "the action declares no unit");
}

#[test]
fn the_return_must_correspond() {
    // the action has no return
    let none = mismatch(OP, "  - {name: x, typedBy: ScalarValues::Integer, direction: in, unit: m}\n  - {name: y, typedBy: ScalarValues::Real, direction: out}\n");
    assert_eq!(none.len(), 1, "{none:?}");
    assert!(none[0].contains("return"), "{none:?}");
    // the operation has no returnType but the action returns
    let extra = mismatch(&OP.replace("    returnType: ScalarValues::Boolean\n", ""), ACT);
    assert_eq!(extra.len(), 1, "{extra:?}");
    assert!(extra[0].contains("return"), "{extra:?}");
    // a different return type
    let ty = mismatch(OP, &ACT.replace("ScalarValues::Boolean, direction: return", "ScalarValues::Integer, direction: return"));
    assert_eq!(ty.len(), 1, "{ty:?}");
    assert!(ty[0].contains("return") && ty[0].contains("type"), "{ty:?}");
}

#[test]
fn an_asynchronous_operation_has_no_return() {
    let op = "  - name: notify\n    isAsync: true\n    parameters:\n      - {name: x, typedBy: ScalarValues::Integer, direction: in}\n    realizedBy: Ops::Act\n";
    let m = mismatch(op, "  - {name: x, typedBy: ScalarValues::Integer, direction: in}\n  - {name: r, typedBy: ScalarValues::Boolean, direction: return}\n");
    assert_eq!(m.len(), 1, "{m:?}");
    assert!(m[0].contains("return"), "{m:?}");
    assert!(mismatch(op, "  - {name: x, typedBy: ScalarValues::Integer, direction: in}\n").is_empty());
}

#[test]
fn several_differences_are_several_findings() {
    let m = mismatch(OP, "  - {name: x, typedBy: ScalarValues::Real, direction: out, unit: s}\n  - {name: z, direction: in}\n");
    assert!(m.len() >= 4, "x direction, x type, x unit, y missing, z extra, return missing: {m:?}");
}

#[test]
fn models_that_declare_none_of_it_see_nothing_new() {
    let els = model("  - name: plain\n    parameters:\n      - {name: a, typedBy: ScalarValues::Integer, direction: in}\n    returnType: ScalarValues::Boolean\n", "  - {name: whatever, direction: in}\n");
    for c in ["E670", "E671", "E672"] {
        assert!(codes(&els, c).is_empty(), "{c}");
    }
}
