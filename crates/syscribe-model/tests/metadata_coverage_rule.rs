//! TC-TRS-META-053 / REQ-TRS-META-053 (GH #379): a stereotype selector in `[[coverage.rule]]`.
//!
//! `stereotype = ["Optional", "Safety where integrity >= 2"]` (any of) selects the requirements a
//! coverage rule applies to, specialisations included. Written before the implementation.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use syscribe_model::config::{CoveragePolicy, ParentRule, ValidateConfig};
use syscribe_model::element::RawElement;
use syscribe_model::resolver::Resolver;
use syscribe_model::validator::validate_with_config;
use syscribe_model::walker::walk_model;

fn write(root: &Path, rel: &str, content: &str) {
    let path = root.join(rel);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, content).unwrap();
}

fn tempdir() -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let p = std::env::temp_dir().join(format!("syscribe-metacov-{}-{}", std::process::id(), N.fetch_add(1, Ordering::Relaxed)));
    std::fs::create_dir_all(&p).unwrap();
    p
}

fn req(root: &Path, id: &str, extra: &str) {
    write(root, &format!("R/{id}.md"), &format!("---\ntype: Requirement\nid: {id}\nname: r\nstatus: approved\nreqDomain: software\n{extra}---\n\nThe system shall work.\n"));
}

/// Requirements: `REQ-CV-001` Optional, `REQ-CV-002` Safety(3), `REQ-CV-003` Safety(1), `REQ-CV-004` ASIL_D(4) (a
/// Safety), `REQ-CV-005` none. `toml` is the `.syscribe.toml`.
fn model(toml: &str) -> (PathBuf, Vec<RawElement>) {
    let r = tempdir();
    write(&r, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(&r, "B/_index.md", "---\ntype: Package\nname: B\n---\n");
    write(&r, "B/Optional.md", "---\ntype: MetadataDef\nname: Optional\n---\n");
    write(&r, "B/Safety.md", "---\ntype: MetadataDef\nname: Safety\nfeatures:\n  - name: integrity\n    typedBy: ScalarValues::Integer\n---\n");
    write(&r, "B/ASIL_D.md", "---\ntype: MetadataDef\nname: ASIL_D\nsupertype: B::Safety\n---\n");
    req(&r, "REQ-CV-001", "metadata:\n  - B::Optional\n");
    req(&r, "REQ-CV-002", "metadata:\n  - type: B::Safety\n    integrity: 3\n");
    req(&r, "REQ-CV-003", "metadata:\n  - type: B::Safety\n    integrity: 1\n");
    req(&r, "REQ-CV-004", "metadata:\n  - type: B::ASIL_D\n    integrity: 4\n");
    req(&r, "REQ-CV-005", "");
    write(&r, ".syscribe.toml", toml);
    let els = walk_model(&r).unwrap();
    (r, els)
}

fn rule(root: &Path, els: &[RawElement], id: &str) -> (ParentRule, String) {
    let policy = CoveragePolicy::load(root);
    let resolver = Resolver::new(els);
    let e = els.iter().find(|e| e.frontmatter.id.as_deref() == Some(id)).unwrap();
    policy.rule_for_element(els, &resolver, e)
}

#[test]
fn a_stereotype_selector_picks_the_rule_for_the_requirements_that_apply_it() {
    let (r, els) = model("[coverage]\n[[coverage.rule]]\nstereotype = [\"Optional\"]\nparent_rule = \"rollup\"\n");
    assert_eq!(rule(&r, &els, "REQ-CV-001"), (ParentRule::Rollup, "rule #1".into()));
    assert_eq!(rule(&r, &els, "REQ-CV-005"), (ParentRule::Both, "default".into()));
}

#[test]
fn a_where_condition_and_specialisation_are_honoured() {
    let (r, els) = model("[coverage]\n[[coverage.rule]]\nstereotype = [\"Safety where integrity >= 2\"]\nparent_rule = \"direct\"\n");
    assert_eq!(rule(&r, &els, "REQ-CV-002").0, ParentRule::Direct, "integrity 3");
    assert_eq!(rule(&r, &els, "REQ-CV-003").0, ParentRule::Both, "integrity 1");
    assert_eq!(rule(&r, &els, "REQ-CV-004").0, ParentRule::Direct, "an ASIL_D is a Safety");
}

#[test]
fn a_list_is_any_of_and_the_first_matching_rule_wins() {
    let (r, els) = model("[coverage]\n[[coverage.rule]]\nstereotype = [\"Optional\", \"Safety\"]\nparent_rule = \"rollup\"\n[[coverage.rule]]\nstereotype = [\"Safety\"]\nparent_rule = \"direct\"\n");
    assert_eq!(rule(&r, &els, "REQ-CV-001"), (ParentRule::Rollup, "rule #1".into()));
    assert_eq!(rule(&r, &els, "REQ-CV-002"), (ParentRule::Rollup, "rule #1".into()), "rule #1 matches first");
}

#[test]
fn it_is_anded_with_the_other_selectors() {
    let (r, els) = model("[coverage]\n[[coverage.rule]]\nstereotype = [\"Safety\"]\ntag = [\"nope\"]\nparent_rule = \"direct\"\n");
    assert_eq!(rule(&r, &els, "REQ-CV-002").0, ParentRule::Both);
}

#[test]
fn a_malformed_selector_makes_the_policy_unusable() {
    for bad in ["stereotype = [\"Safety where integrity >=\"]", "stereotype = [\"\"]", "stereotype = [1]", "stereotype = []"] {
        let (r, _) = model(&format!("[coverage]\n[[coverage.rule]]\n{bad}\nparent_rule = \"direct\"\n"));
        let p = CoveragePolicy::load(&r);
        assert!(p.problems.iter().any(|m| m.contains("stereotype")), "{bad}: {:?}", p.problems);
    }
}

#[test]
fn a_stereotype_rule_cannot_loosen_an_integrity_rated_parent() {
    // REQ-CV-006 is ASIL B and a parent (REQ-CV-007 derives from it); the Optional rule says `rollup`.
    let (r, _) = model("[coverage]\n[[coverage.rule]]\nstereotype = [\"Optional\"]\nparent_rule = \"rollup\"\n");
    write(&r, "A/ADR-CV-001.md", "---\ntype: ADR\nid: ADR-CV-001\nname: a\nstatus: accepted\n---\n\nA.\n");
    req(&r, "REQ-CV-006", "asilLevel: B\nmetadata:\n  - B::Optional\n");
    req(&r, "REQ-CV-007", "derivedFrom: [REQ-CV-006]\nbreakdownAdr: ADR-CV-001\n");
    let els = walk_model(&r).unwrap();
    let f = validate_with_config(&els, &ValidateConfig::with_model_root(&r)).findings;
    assert!(f.iter().any(|x| x.code == "E898" && x.file.contains("REQ-CV-006")), "{f:?}");
}
