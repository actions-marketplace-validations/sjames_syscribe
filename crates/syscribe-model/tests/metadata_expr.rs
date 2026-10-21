//! TC-TRS-META-023 / REQ-TRS-META-023 (GH #376): `has(Def)` and `meta(Def).key` in the
//! expression language, evaluated against the element in scope.
//!
//! They are exercised through the `constraints:` of a second stereotype, `Regulatory`, whose
//! carrier may or may not also apply `Safety`. Written before the implementation.

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
    let p = std::env::temp_dir().join(format!("syscribe-metaexpr-{}-{}", std::process::id(), N.fetch_add(1, Ordering::Relaxed)));
    std::fs::create_dir_all(&p).unwrap();
    p
}

/// `Regulatory` carries `constraint`; `B::Item` applies `application` (YAML list items).
fn model(constraint: &str, application: &str) -> Vec<RawElement> {
    let r = tempdir();
    write(&r, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(&r, "B/_index.md", "---\ntype: Package\nname: B\n---\n");
    write(
        &r,
        "B/Safety.md",
        "---\ntype: MetadataDef\nname: Safety\nfeatures:\n  - name: integrity\n    typedBy: ScalarValues::Integer\n  - name: level\n    typedBy: ScalarValues::Integer\n    default: 3\n---\n",
    );
    write(
        &r,
        "B/Regulatory.md",
        &format!("---\ntype: MetadataDef\nname: Regulatory\nfeatures:\n  - name: status\n    typedBy: ScalarValues::String\nconstraints:\n  - '{constraint}'\n---\n"),
    );
    write(&r, "B/Item.md", &format!("---\ntype: PartDef\nname: Item\nmetadata:\n{application}---\n"));
    walk_model(&r).unwrap()
}

fn codes(els: &[RawElement], code: &str) -> Vec<String> {
    validate(els).findings.iter().filter(|f| f.code == code).map(|f| f.message.clone()).collect()
}

const REG: &str = "  - type: B::Regulatory\n    status: unverified\n";

#[test]
fn has_is_true_when_the_element_applies_the_stereotype_and_false_otherwise() {
    let with = model("has(Safety)", &format!("{REG}  - type: B::Safety\n    integrity: 1\n"));
    assert!(codes(&with, "E327").is_empty(), "{:?}", validate(&with).findings);
    let without = model("has(Safety)", REG);
    assert_eq!(codes(&without, "E327").len(), 1, "has(Safety) is false without the stereotype");
}

#[test]
fn meta_reads_a_tagged_value() {
    let ok = model("meta(Safety).integrity >= 2", &format!("{REG}  - type: B::Safety\n    integrity: 3\n"));
    assert!(codes(&ok, "E327").is_empty(), "{:?}", validate(&ok).findings);
    let low = model("meta(Safety).integrity >= 2", &format!("{REG}  - type: B::Safety\n    integrity: 1\n"));
    assert_eq!(codes(&low, "E327").len(), 1);
}

#[test]
fn meta_yields_the_declared_default_when_the_value_is_not_written() {
    let els = model("meta(Safety).level == 3", &format!("{REG}  - type: B::Safety\n    integrity: 1\n"));
    assert!(codes(&els, "E327").is_empty(), "{:?}", validate(&els).findings);
}

#[test]
fn an_absent_stereotype_is_a_clean_absent_value_not_an_evaluation_error() {
    // The comparison is false, so the constraint is violated — an ordinary E327, no parse or
    // evaluation finding, and no panic.
    let els = model("meta(Safety).integrity >= 2", REG);
    assert_eq!(codes(&els, "E327").len(), 1);
    let msg = &codes(&els, "E327")[0];
    assert!(msg.contains("violated"), "{msg}");
    // Guarded by has(), the same element is fine.
    let guarded = model("not has(Safety) or meta(Safety).integrity >= 2", REG);
    assert!(codes(&guarded, "E327").is_empty());
}

#[test]
fn an_absent_value_compares_false_both_ways() {
    // integrity is not written: neither `>= 0` nor `< 0` holds.
    let ge = model("meta(Safety).integrity >= 0", &format!("{REG}  - B::Safety\n"));
    let lt = model("meta(Safety).integrity < 0", &format!("{REG}  - B::Safety\n"));
    assert_eq!(codes(&ge, "E327").len(), 1);
    assert_eq!(codes(&lt, "E327").len(), 1);
}

#[test]
fn a_malformed_call_is_reported_on_the_definition_not_the_carrier() {
    let els = model("meta(Safety)", &format!("{REG}  - type: B::Safety\n    integrity: 1\n"));
    let f: Vec<_> = validate(&els).findings.into_iter().filter(|f| f.code == "E327").collect();
    assert_eq!(f.len(), 1, "{f:?}");
    assert!(f[0].file.contains("Regulatory.md"), "{:?}", f[0].file);
}

#[test]
fn has_with_an_unknown_definition_is_reported_on_the_definition() {
    let els = model("has(Nonesuch)", REG);
    let f: Vec<_> = validate(&els).findings.into_iter().filter(|f| f.code == "E327").collect();
    assert!(f.iter().any(|f| f.file.contains("Regulatory.md") && f.message.contains("Nonesuch")), "{f:?}");
}
