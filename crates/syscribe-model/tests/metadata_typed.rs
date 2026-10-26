//! TC-TRS-META-010, -011, -013, -014 / REQ-TRS-META-010..014 (GH #375): typed tagged values.
//!
//! Written before the implementation: each test fails until `validate` checks an application's
//! tagged values against the typing declared on its `MetadataDef` features.

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
    let p = std::env::temp_dir().join(format!("syscribe-metatyped-{}-{}", std::process::id(), N.fetch_add(1, Ordering::Relaxed)));
    std::fs::create_dir_all(&p).unwrap();
    p
}

/// A model with `B::Safety` defined by `def_features` (YAML `features:` list items, already
/// indented) and one `B::Item` PartDef that applies it with `application` (YAML list items).
fn model(def_features: &str, application: &str) -> Vec<RawElement> {
    model_with(def_features, "", application)
}

fn model_with(def_features: &str, def_extra: &str, application: &str) -> Vec<RawElement> {
    let r = tempdir();
    write(&r, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(&r, "B/_index.md", "---\ntype: Package\nname: B\n---\n");
    write(
        &r,
        "B/Safety.md",
        &format!("---\ntype: MetadataDef\nname: Safety\nfeatures:\n{def_features}{def_extra}---\n"),
    );
    write(&r, "B/Other.md", "---\ntype: PartDef\nname: Other\n---\n");
    write(&r, "B/Item.md", &format!("---\ntype: PartDef\nname: Item\nmetadata:\n{application}---\n"));
    walk_model(&r).unwrap()
}

fn codes(els: &[RawElement], code: &str) -> Vec<String> {
    validate(els).findings.iter().filter(|f| f.code == code).map(|f| f.message.clone()).collect()
}

const INTEGRITY: &str = "  - name: integrity\n    typedBy: ScalarValues::Integer\n    range: \"0..4\"\n";

// ── REQ-TRS-META-010 ───────────────────────────────────────────────────────

#[test]
fn a_value_of_the_wrong_scalar_type_is_e322() {
    let els = model(INTEGRITY, "  - type: B::Safety\n    integrity: banana\n");
    let m = codes(&els, "E322");
    assert_eq!(m.len(), 1, "{m:?}");
    assert!(m[0].contains("Safety") && m[0].contains("integrity"), "names stereotype and key: {m:?}");
}

#[test]
fn a_unit_of_a_different_dimension_is_e322() {
    let els = model(
        "  - name: mass\n    typedBy: ScalarValues::Real\n    unit: kg\n",
        "  - type: B::Safety\n    mass: \"3 s\"\n",
    );
    assert_eq!(codes(&els, "E322").len(), 1, "{:?}", validate(&els).findings);
}

#[test]
fn a_value_outside_the_range_is_e323() {
    let els = model(INTEGRITY, "  - type: B::Safety\n    integrity: 7\n");
    assert_eq!(codes(&els, "E323").len(), 1);
    assert!(codes(&els, "E322").is_empty(), "7 is an Integer");
}

#[test]
fn a_value_inside_the_range_is_quiet() {
    let els = model(INTEGRITY, "  - type: B::Safety\n    integrity: 4\n");
    for c in ["E322", "E323", "E324", "E325"] {
        assert!(codes(&els, c).is_empty(), "{c}");
    }
}

#[test]
fn a_value_not_in_the_enumeration_is_e324() {
    let def = "  - name: status\n    typedBy: ScalarValues::String\n    enumValues: [verified, unverified]\n";
    let bad = model(def, "  - type: B::Safety\n    status: maybe\n");
    assert_eq!(codes(&bad, "E324").len(), 1);
    let good = model(def, "  - type: B::Safety\n    status: verified\n");
    assert!(codes(&good, "E324").is_empty());
}

#[test]
fn undeclared_typing_and_bare_references_are_unchanged() {
    // A feature with no typing: only the existing W045 name check applies.
    let els = model("  - name: note\n", "  - type: B::Safety\n    note: anything\n  - B::Safety\n");
    for c in ["E322", "E323", "E324", "E325", "E326", "E327"] {
        assert!(codes(&els, c).is_empty(), "{c}");
    }
    assert!(codes(&els, "W045").is_empty());
}

// ── REQ-TRS-META-011 ───────────────────────────────────────────────────────

#[test]
fn an_omitted_required_value_is_e325() {
    let def = format!("{INTEGRITY}  - name: goal\n    typedBy: ScalarValues::String\n    multiplicity: \"1\"\n");
    let els = model(&def, "  - type: B::Safety\n    integrity: 1\n");
    let m = codes(&els, "E325");
    assert_eq!(m.len(), 1, "{m:?}");
    assert!(m[0].contains("goal"), "{m:?}");
}

#[test]
fn too_many_values_for_the_upper_bound_is_e325() {
    let def = "  - name: goal\n    typedBy: ScalarValues::String\n    multiplicity: \"0..1\"\n";
    let els = model(def, "  - type: B::Safety\n    goal: [a, b]\n");
    assert_eq!(codes(&els, "E325").len(), 1);
}

#[test]
fn a_list_is_accepted_when_the_upper_bound_allows_it() {
    let def = "  - name: goal\n    typedBy: ScalarValues::String\n    multiplicity: \"1..*\"\n";
    let els = model(def, "  - type: B::Safety\n    goal: [a, b]\n");
    assert!(codes(&els, "E325").is_empty());
}

#[test]
fn a_feature_without_multiplicity_is_optional() {
    let els = model("  - name: goal\n    typedBy: ScalarValues::String\n", "  - type: B::Safety\n");
    assert!(codes(&els, "E325").is_empty());
}

#[test]
fn a_required_value_with_a_default_is_not_e325() {
    let def = "  - name: level\n    typedBy: ScalarValues::Integer\n    multiplicity: \"1\"\n    default: 2\n";
    let els = model(def, "  - type: B::Safety\n");
    assert!(codes(&els, "E325").is_empty());
}

// ── REQ-TRS-META-012 (the definition side; visibility is in metadata_typed_cli.rs) ──

#[test]
fn a_default_that_violates_its_own_type_is_reported_on_the_definition() {
    let def = "  - name: level\n    typedBy: ScalarValues::Integer\n    default: x\n";
    let els = model(def, "  - B::Safety\n");
    let f: Vec<_> = validate(&els).findings.into_iter().filter(|f| f.code == "E322").collect();
    assert_eq!(f.len(), 1, "{f:?}");
    assert!(f[0].file.contains("Safety.md"), "{:?}", f[0].file);
}

// ── REQ-TRS-META-013 ───────────────────────────────────────────────────────

const REVIEWER: &str = "  - name: reviewer\n    typedBy: PartDef\n";

#[test]
fn a_reference_that_resolves_is_quiet() {
    let els = model(REVIEWER, "  - type: B::Safety\n    reviewer: B::Other\n");
    assert!(codes(&els, "E326").is_empty());
}

#[test]
fn an_unresolved_reference_is_e326() {
    let els = model(REVIEWER, "  - type: B::Safety\n    reviewer: B::Nobody\n");
    assert_eq!(codes(&els, "E326").len(), 1);
}

#[test]
fn a_reference_to_the_wrong_element_kind_is_e326() {
    // B::Safety is a MetadataDef, not a PartDef.
    let els = model(REVIEWER, "  - type: B::Safety\n    reviewer: B::Safety\n");
    assert_eq!(codes(&els, "E326").len(), 1);
}

#[test]
fn a_tagged_value_reference_makes_its_holder_a_referrer_of_the_target() {
    let els = model(REVIEWER, "  - type: B::Safety\n    reviewer: B::Other\n");
    let r = syscribe_model::mutate::referrers(&els, "B::Other");
    assert!(r.iter().any(|(q, _)| q == "B::Item"), "delete_element must see it: {r:?}");
}

// ── REQ-TRS-META-014 ───────────────────────────────────────────────────────

const CONSTRAINED: &str = "  - name: integrity\n    typedBy: ScalarValues::Integer\n  - name: goal\n    typedBy: ScalarValues::String\n";
const CONSTRAINT: &str = "constraints:\n  - 'integrity >= 2 implies goal != \"\"'\n";

#[test]
fn a_violated_constraint_is_e327_naming_definition_expression_and_element() {
    let els = model_with(CONSTRAINED, CONSTRAINT, "  - type: B::Safety\n    integrity: 3\n");
    let m = codes(&els, "E327");
    assert_eq!(m.len(), 1, "{m:?}");
    assert!(m[0].contains("Safety") && m[0].contains("integrity >= 2"), "{m:?}");
}

#[test]
fn a_satisfied_constraint_is_quiet() {
    let els = model_with(CONSTRAINED, CONSTRAINT, "  - type: B::Safety\n    integrity: 3\n    goal: G-1\n");
    assert!(codes(&els, "E327").is_empty());
}

#[test]
fn a_constraint_over_an_absent_optional_value_is_not_a_violation() {
    let els = model_with(CONSTRAINED, CONSTRAINT, "  - type: B::Safety\n    integrity: 1\n");
    assert!(codes(&els, "E327").is_empty());
    let none = model_with(CONSTRAINED, CONSTRAINT, "  - B::Safety\n");
    assert!(codes(&none, "E327").is_empty(), "a bare application");
}

#[test]
fn an_unparsable_constraint_is_reported_on_the_definition() {
    let els = model_with(CONSTRAINED, "constraints:\n  - 'integrity >= '\n", "  - type: B::Safety\n    integrity: 3\n");
    let f: Vec<_> = validate(&els).findings.into_iter().filter(|f| f.code == "E327").collect();
    assert_eq!(f.len(), 1, "{f:?}");
    assert!(f[0].file.contains("Safety.md"), "{:?}", f[0].file);
}
