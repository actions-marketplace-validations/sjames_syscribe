//! TC-TRS-OPCALL-001..002 / REQ-TRS-OPCALL-001..002 (GH #364 item 3, #398): `operation:` on a sub-action and the
//! check of its arguments against the operation's parameters.
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
    let p = std::env::temp_dir().join(format!("syscribe-opcall-{}-{}", std::process::id(), N.fetch_add(1, Ordering::Relaxed)));
    std::fs::create_dir_all(&p).unwrap();
    p
}

const IFACE: &str = "operations:
  - name: read
    returnType: ScalarValues::Real
    parameters:
      - {name: channel, typedBy: ScalarValues::Integer, direction: in}
  - name: calibrate
    isAsync: true
    parameters:
      - {name: gain, typedBy: ScalarValues::Real, direction: in}
      - {name: offset, typedBy: ScalarValues::Real, direction: in}
";

/// An `Api` interface with `IFACE` and an action whose `subActions:` list is `subs`.
fn model(subs: &str) -> Vec<RawElement> {
    let r = tempdir();
    write(&r, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(&r, "Api.md", &format!("---\ntype: InterfaceDef\nname: Api\n{IFACE}---\n\nApi.\n"));
    write(&r, "Act.md", &format!("---\ntype: ActionDef\nname: Act\nsubActions:\n{subs}---\n\nAct.\n"));
    walk_model(&r).unwrap()
}

fn codes(els: &[RawElement], code: &str) -> Vec<(String, String)> {
    validate(els).findings.iter().filter(|f| f.code == code).map(|f| (f.file.clone(), f.message.clone())).collect()
}

fn call(kind: &str, op: &str, args: &str) -> String {
    format!("  - name: c\n    kind: {kind}\n    operation: {op}\n    parameters:\n{args}")
}

// ── REQ-TRS-OPCALL-001 ─────────────────────────────────────────────────────

#[test]
fn a_call_to_a_declared_operation_is_clean() {
    for kind in ["SendAction", "PerformAction"] {
        let els = model(&call(kind, "Api::read", "      - {name: channel, value: \"1\"}\n"));
        assert!(codes(&els, "E673").is_empty() && codes(&els, "E674").is_empty(), "{kind}: {:?}", validate(&els).findings);
    }
}

#[test]
fn an_unknown_owner_or_operation_is_e673() {
    let owner = codes(&model(&call("SendAction", "Nope::read", "      - {name: channel, value: \"1\"}\n")), "E673");
    assert_eq!(owner.len(), 1, "{owner:?}");
    assert!(owner[0].1.contains("Nope::read"), "{owner:?}");
    let op = codes(&model(&call("SendAction", "Api::write", "")), "E673");
    assert_eq!(op.len(), 1, "{op:?}");
    assert!(op[0].1.contains("write"), "{op:?}");
}

#[test]
fn operation_on_another_kind_of_sub_action_is_e673() {
    let els = model("  - name: c\n    kind: AssignmentAction\n    operation: Api::read\n");
    assert_eq!(codes(&els, "E673").len(), 1);
}

#[test]
fn a_nested_call_is_found() {
    let els = model("  - name: i\n    kind: IfAction\n    condition: true\n    then:\n      - name: c\n        kind: SendAction\n        operation: Api::nope\n");
    assert_eq!(codes(&els, "E673").len(), 1);
}

#[test]
fn a_sub_action_without_operation_is_not_looked_at() {
    let els = model("  - name: c\n    kind: SendAction\n    payload: Api\n");
    assert!(codes(&els, "E673").is_empty() && codes(&els, "E674").is_empty());
}

// ── REQ-TRS-OPCALL-002 ─────────────────────────────────────────────────────

#[test]
fn an_unknown_argument_is_e674() {
    let els = model(&call("SendAction", "Api::read", "      - {name: channel, value: \"1\"}\n      - {name: speed, value: \"2\"}\n"));
    let e = codes(&els, "E674");
    assert_eq!(e.len(), 1, "{e:?}");
    assert!(e[0].1.contains("speed"), "{e:?}");
}

#[test]
fn a_missing_in_parameter_is_e674() {
    let els = model(&call("SendAction", "Api::calibrate", "      - {name: gain, value: \"1\"}\n"));
    let e = codes(&els, "E674");
    assert_eq!(e.len(), 1, "{e:?}");
    assert!(e[0].1.contains("offset"), "{e:?}");
}

#[test]
fn all_in_parameters_given_is_clean() {
    let els = model(&call("SendAction", "Api::calibrate", "      - {name: gain, value: \"1\"}\n      - {name: offset, value: \"0\"}\n"));
    assert!(codes(&els, "E674").is_empty());
}
