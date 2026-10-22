//! TC-TRS-UNIT-001..002 / REQ-TRS-UNIT-001..002 (GH #371, slice one): `level: component | unit` and the checks on a unit.
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
    let p = std::env::temp_dir().join(format!("syscribe-unit-{}-{}", std::process::id(), N.fetch_add(1, Ordering::Relaxed)));
    std::fs::create_dir_all(&p).unwrap();
    p
}

const UNIT: &str = "domain: software\nlevel: unit\nstatus: approved\nimplementedBy: \"repo:src/lib.rs#parse\"\nsatisfies:\n  - REQ-001\n";
const COMP: &str = "domain: software\nlevel: component\nfeatures:\n  - {name: u, typedBy: Sw::Unit}\n";
const TEST: &str = "---\ntype: TestCase\nid: TC-001\nname: t\nstatus: draft\ntestLevel: L1\nverifies:\n  - REQ-001\n---\n\n```gherkin\nFeature: f\n  Scenario: s\n    Then ok\n```\n";

/// A model with a requirement, a `Sw::Unit` carrying `unit`, a component `Sw::Comp` carrying `comp`, and optionally a test.
fn model(unit: &str, comp: &str, test: Option<&str>) -> Vec<RawElement> {
    let r = tempdir();
    write(&r, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(&r, "Sw/_index.md", "---\ntype: Package\nname: Sw\n---\n");
    write(&r, "REQ-001.md", "---\ntype: Requirement\nid: REQ-001\nname: r\nstatus: draft\nreqDomain: software\n---\n\nThe unit shall parse.\n");
    write(&r, "Sw/Unit.md", &format!("---\ntype: PartDef\nname: Unit\n{unit}---\n\nUnit.\n"));
    write(&r, "Sw/Comp.md", &format!("---\ntype: PartDef\nname: Comp\n{comp}---\n\nComp.\n"));
    if let Some(t) = test {
        write(&r, "TC-001.md", t);
    }
    walk_model(&r).unwrap()
}

fn codes(els: &[RawElement], code: &str) -> Vec<(String, String)> {
    validate(els).findings.iter().filter(|f| f.code == code).map(|f| (f.file.clone(), f.message.clone())).collect()
}

// ── REQ-TRS-UNIT-001 ───────────────────────────────────────────────────────

#[test]
fn a_complete_unit_is_clean() {
    let els = model(UNIT, COMP, Some(TEST));
    for c in ["E740", "E741", "E742", "E743", "E744", "W740", "W047"] {
        assert!(codes(&els, c).is_empty(), "{c}: {:?}", codes(&els, c));
    }
}

#[test]
fn an_unknown_level_is_e740() {
    let els = model("domain: software\nlevel: module\n", COMP, Some(TEST));
    let e = codes(&els, "E740");
    assert_eq!(e.len(), 1, "{e:?}");
    assert!(e[0].1.contains("module"), "{e:?}");
}

#[test]
fn level_on_another_element_is_w047() {
    let r = tempdir();
    write(&r, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(&r, "Hw.md", "---\ntype: ItemDef\nname: Hw\nlevel: unit\n---\n\nx\n");
    let els = walk_model(&r).unwrap();
    assert!(codes(&els, "W047").iter().any(|(_, m)| m.contains("level")));
}

#[test]
fn a_component_or_unlevelled_part_is_not_checked() {
    let els = model("domain: software\nstatus: approved\n", "domain: software\nlevel: component\nstatus: approved\n", None);
    for c in ["E741", "E742", "E743", "E744"] {
        assert!(codes(&els, c).is_empty(), "{c}");
    }
}

// ── REQ-TRS-UNIT-002 ───────────────────────────────────────────────────────

#[test]
fn a_unit_without_code_is_e741() {
    let els = model("domain: software\nlevel: unit\nstatus: approved\nsatisfies:\n  - REQ-001\n", COMP, Some(TEST));
    assert_eq!(codes(&els, "E741").len(), 1);
}

#[test]
fn code_without_a_symbol_is_w740() {
    let els = model(&UNIT.replace("#parse", ""), COMP, Some(TEST));
    assert_eq!(codes(&els, "W740").len(), 1);
    assert!(codes(&els, "E741").is_empty());
}

#[test]
fn a_unit_that_satisfies_nothing_is_e742() {
    let els = model(&UNIT.replace("satisfies:\n  - REQ-001\n", ""), COMP, Some(TEST));
    assert_eq!(codes(&els, "E742").len(), 1);
}

#[test]
fn a_unit_without_an_l1_test_is_e743() {
    assert_eq!(codes(&model(UNIT, COMP, None), "E743").len(), 1);
    let l2 = TEST.replace("testLevel: L1", "testLevel: L2");
    assert_eq!(codes(&model(UNIT, COMP, Some(&l2)), "E743").len(), 1, "an L2 test is not a unit test");
}

#[test]
fn a_test_that_verifies_the_unit_itself_counts() {
    let t = TEST.replace("REQ-001", "Sw::Unit");
    assert!(codes(&model(UNIT, COMP, Some(&t)), "E743").is_empty());
}

#[test]
fn a_unit_outside_any_component_is_e744() {
    let els = model(UNIT, "domain: software\nlevel: component\n", Some(TEST));
    assert_eq!(codes(&els, "E744").len(), 1);
    let not_comp = model(UNIT, "domain: software\nfeatures:\n  - {name: u, typedBy: Sw::Unit}\n", Some(TEST));
    assert_eq!(codes(&not_comp, "E744").len(), 1, "the container must be level: component");
}

#[test]
fn a_draft_unit_is_not_checked() {
    let els = model("domain: software\nlevel: unit\nstatus: draft\n", "domain: software\n", None);
    for c in ["E741", "E742", "E743", "E744"] {
        assert!(codes(&els, c).is_empty(), "{c}");
    }
}
