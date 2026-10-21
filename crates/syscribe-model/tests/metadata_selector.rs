//! TC-TRS-META-024 / REQ-TRS-META-024 (GH #376, #379): the common stereotype selector
//! `{stereotype: Def, where: <expr>}` and its use in a TestPlan's `selection:`.
//!
//! Written before the implementation.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use syscribe_model::element::RawElement;
use syscribe_model::resolver::Resolver;
use syscribe_model::tagged_values::{parse_selector, selector_matches};
use syscribe_model::testplan::effective_testcases;
use syscribe_model::validator::validate;
use syscribe_model::walker::walk_model;

fn write(root: &Path, rel: &str, content: &str) {
    let path = root.join(rel);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, content).unwrap();
}

fn tempdir() -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let p = std::env::temp_dir().join(format!("syscribe-metasel-{}-{}", std::process::id(), N.fetch_add(1, Ordering::Relaxed)));
    std::fs::create_dir_all(&p).unwrap();
    p
}

/// `Safety` (integrity) and `ASIL_D :> Safety`. Parts: `P1` Safety(3), `P2` Safety(1), `P3` ASIL_D(4), `P4` none.
/// Requirements `REQ-SEL-001` Safety(3) and `REQ-SEL-002` none; TestCases `TC-SEL-001` verifies 001,
/// `TC-SEL-002` verifies 002, `TC-SEL-003` applies Safety(2) itself, `TC-SEL-004` (draft) verifies 001.
fn model(plan_selection: &str) -> Vec<RawElement> {
    let r = tempdir();
    write(&r, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(&r, "B/_index.md", "---\ntype: Package\nname: B\n---\n");
    write(&r, "B/Safety.md", "---\ntype: MetadataDef\nname: Safety\nfeatures:\n  - name: integrity\n    typedBy: ScalarValues::Integer\n---\n");
    write(&r, "B/ASIL_D.md", "---\ntype: MetadataDef\nname: ASIL_D\nsupertype: B::Safety\n---\n");
    let part = |n: &str, m: &str| write(&r, &format!("B/{n}.md"), &format!("---\ntype: PartDef\nname: {n}\n{m}---\n\n{n}.\n"));
    part("P1", "metadata:\n  - type: B::Safety\n    integrity: 3\n");
    part("P2", "metadata:\n  - type: B::Safety\n    integrity: 1\n");
    part("P3", "metadata:\n  - type: B::ASIL_D\n    integrity: 4\n");
    part("P4", "");
    let req = |id: &str, m: &str| write(&r, &format!("R/{id}.md"), &format!("---\ntype: Requirement\nid: {id}\nname: r\nstatus: approved\nreqDomain: software\n{m}---\n\nThe system shall work.\n"));
    req("REQ-SEL-001", "metadata:\n  - type: B::Safety\n    integrity: 3\n");
    req("REQ-SEL-002", "");
    let tc = |id: &str, status: &str, verifies: &str, m: &str| {
        write(&r, &format!("T/{id}.md"), &format!("---\ntype: TestCase\nid: {id}\nname: t\nstatus: {status}\ntestLevel: L2\nverifies: [{verifies}]\n{m}---\n\n```gherkin\nScenario: s\n  Given a\n  Then b\n```\n"))
    };
    tc("TC-SEL-001", "active", "REQ-SEL-001", "");
    tc("TC-SEL-002", "active", "REQ-SEL-002", "");
    tc("TC-SEL-003", "active", "REQ-SEL-002", "metadata:\n  - type: B::Safety\n    integrity: 2\n");
    tc("TC-SEL-004", "draft", "REQ-SEL-001", "");
    write(&r, "P/TP-SEL-001.md", &format!("---\ntype: TestPlan\nid: TP-SEL-001\nname: plan\nstatus: approved\nscope: unit\nselection:\n{plan_selection}---\n\nPlan.\n"));
    walk_model(&r).unwrap()
}

fn find<'a>(els: &'a [RawElement], q: &str) -> &'a RawElement {
    els.iter().find(|e| e.qualified_name == q || e.frontmatter.id.as_deref() == Some(q)).unwrap_or_else(|| panic!("missing {q}"))
}

fn y(s: &str) -> serde_yaml::Value {
    serde_yaml::from_str(s).unwrap()
}

fn members(els: &[RawElement]) -> Vec<String> {
    let resolver = Resolver::new(els);
    let plan = find(els, "TP-SEL-001");
    let mut v: Vec<String> = effective_testcases(plan, els, &resolver).iter().filter_map(|t| t.frontmatter.id.clone()).collect();
    v.sort();
    v
}

fn codes(els: &[RawElement], code: &str) -> Vec<(String, String)> {
    validate(els).findings.iter().filter(|f| f.code == code).map(|f| (f.file.clone(), f.message.clone())).collect()
}

// ── the grammar ────────────────────────────────────────────────────────────

#[test]
fn a_selector_is_a_mapping_or_a_shorthand_string() {
    let a = parse_selector(&y("{stereotype: Safety, where: 'integrity >= 2'}")).unwrap();
    let b = parse_selector(&y("'Safety where integrity >= 2'")).unwrap();
    assert_eq!((a.def.as_str(), a.when.as_deref()), ("Safety", Some("integrity >= 2")));
    assert_eq!((b.def.as_str(), b.when.as_deref()), ("Safety", Some("integrity >= 2")));
    assert_eq!(parse_selector(&y("Safety")).unwrap().when, None);
    assert_eq!(parse_selector(&y("{stereotype: Safety}")).unwrap().when, None);
}

#[test]
fn a_malformed_selector_says_why() {
    assert!(parse_selector(&y("{where: 'x > 1'}")).unwrap_err().contains("stereotype"));
    assert!(parse_selector(&y("{stereotype: Safety, where: 'integrity >='}")).unwrap_err().contains("where"));
    assert!(parse_selector(&y("{stereotype: Safety, bogus: 1}")).unwrap_err().contains("bogus"));
    assert!(parse_selector(&y("[1, 2]")).is_err());
}

#[test]
fn a_selector_matches_the_elements_that_apply_the_stereotype_under_its_condition() {
    let els = model("  tags: [x]\n");
    let resolver = Resolver::new(&els);
    let sel = parse_selector(&y("{stereotype: Safety, where: 'integrity >= 2'}")).unwrap();
    let hit = |q: &str| selector_matches(&els, &resolver, find(&els, q), &sel);
    assert!(hit("B::P1"));
    assert!(!hit("B::P2"), "integrity 1");
    assert!(hit("B::P3"), "an ASIL_D is a Safety, integrity 4");
    assert!(!hit("B::P4"), "no stereotype");
    let any = parse_selector(&y("Safety")).unwrap();
    assert!(selector_matches(&els, &resolver, find(&els, "B::P2"), &any));
    let sub = parse_selector(&y("ASIL_D")).unwrap();
    assert!(!selector_matches(&els, &resolver, find(&els, "B::P1"), &sub), "a plain Safety is not an ASIL_D");
}

// ── TestPlan selection ─────────────────────────────────────────────────────

#[test]
fn a_plan_selects_test_cases_by_their_own_stereotype_or_that_of_a_requirement_they_verify() {
    let els = model("  stereotypes:\n    - {stereotype: Safety, where: 'integrity >= 2'}\n");
    // TC-SEL-001 verifies REQ-SEL-001 (Safety 3); TC-SEL-003 applies Safety 2 itself;
    // TC-SEL-002 verifies a requirement with none; TC-SEL-004 is a draft, never swept.
    assert_eq!(members(&els), ["TC-SEL-001", "TC-SEL-003"]);
}

#[test]
fn several_selectors_are_any_of_and_compose_with_the_other_selection_fields() {
    let els = model("  stereotypes:\n    - {stereotype: Safety, where: 'integrity > 2'}\n    - 'Safety where integrity == 2'\n");
    assert_eq!(members(&els), ["TC-SEL-001", "TC-SEL-003"]);
    let anded = model("  testLevels: [L3]\n  stereotypes: [Safety]\n");
    assert!(members(&anded).is_empty(), "all present fields must hold");
}

#[test]
fn a_selection_with_only_stereotypes_is_not_empty() {
    let els = model("  stereotypes: [Safety]\n");
    assert!(!members(&els).is_empty());
}

#[test]
fn a_malformed_or_unknown_selector_in_a_selection_is_e651_on_the_plan() {
    let bad_where = model("  stereotypes:\n    - {stereotype: Safety, where: 'integrity >='}\n");
    let unknown = model("  stereotypes: [Nonesuch]\n");
    let junk = model("  stereotypes: [{bogus: 1}]\n");
    for els in [&bad_where, &unknown, &junk] {
        let e = codes(els, "E651");
        assert_eq!(e.len(), 1, "{e:?}");
        assert!(e[0].0.ends_with("TP-SEL-001.md"), "{e:?}");
    }
    assert!(codes(&unknown, "E651")[0].1.contains("Nonesuch"));
}
