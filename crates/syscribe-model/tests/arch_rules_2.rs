//! TC-TRS-ARCH-006..008 / REQ-TRS-ARCH-006..008 (GH #369, slice two): `allow-only`, `layering` and `max-depth`
//! rules, and waivers.
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
    let p = std::env::temp_dir().join(format!("syscribe-arch2-{}-{}", std::process::id(), N.fetch_add(1, Ordering::Relaxed)));
    std::fs::create_dir_all(&p).unwrap();
    p
}

/// Parts `P::A`..`P::D` (tag `chain`, layers ui/domain/platform/none), each with ports `o` (out) and `i` (in), and
/// `P::Top` using all four with `connections` (`from: x.o, to: y.i` makes Y depend on X). `rule` is the rule's
/// front matter body.
fn model(connections: &[(&str, &str)], rule: &str) -> Vec<RawElement> {
    let r = tempdir();
    write(&r, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    for p in ["P", "Rules"] {
        write(&r, &format!("{p}/_index.md"), &format!("---\ntype: Package\nname: {p}\n---\n"));
    }
    for (n, layer) in [("A", "layer: ui\n"), ("B", "layer: domain\n"), ("C", "layer: platform\n"), ("D", "")] {
        write(&r, &format!("P/{n}.md"), &format!("---\ntype: PartDef\nname: {n}\ndomain: software\ntags: [chain]\n{layer}features:\n  - {{name: o, type: Port, direction: out}}\n  - {{name: i, type: Port, direction: in}}\n---\n\n{n}.\n"));
    }
    let conns: String = connections.iter().map(|(f, t)| format!("  - {{from: {f}.o, to: {t}.i}}\n")).collect();
    write(
        &r,
        "P/Top.md",
        &format!("---\ntype: PartDef\nname: Top\ndomain: system\nfeatures:\n  - {{name: a, typedBy: P::A}}\n  - {{name: b, typedBy: P::B}}\n  - {{name: c, typedBy: P::C}}\n  - {{name: d, typedBy: P::D}}\nconnections:\n{conns}---\n\nTop.\n"),
    );
    write(&r, "Rules/AR-ARC-002.md", &format!("---\ntype: ArchitectureRule\nid: AR-ARC-002\nname: the rule\n{rule}---\n\nWhy.\n"));
    walk_model(&r).unwrap()
}

fn codes(els: &[RawElement], code: &str) -> Vec<(String, String)> {
    validate(els).findings.iter().filter(|f| f.code == code).map(|f| (f.file.clone(), f.message.clone())).collect()
}

const ALLOW: &str = "kind: allow-only\nfrom: {layer: ui}\nto: {layer: domain}\n";
const LAYERS: &str = "kind: layering\nlayers: [ui, domain, platform]\n";

// ── REQ-TRS-ARCH-006 ───────────────────────────────────────────────────────

#[test]
fn allow_only_accepts_the_allowed_and_reports_the_rest() {
    // a consumer depends on its producer: B.o -> A.i makes A depend on B (allowed); C.o -> A.i makes A depend on C
    let ok = model(&[("b", "a")], ALLOW);
    assert!(codes(&ok, "E660").is_empty(), "{:?}", validate(&ok).findings);
    let bad = model(&[("b", "a"), ("c", "a")], ALLOW);
    let e = codes(&bad, "E660");
    assert_eq!(e.len(), 1, "{e:?}");
    assert!(e[0].1.contains("AR-ARC-002") && e[0].1.contains("P::A") && e[0].1.contains("P::C"), "{e:?}");
    assert!(!e[0].1.contains("P::B"), "the allowed one is not named: {e:?}");
}

#[test]
fn allow_only_counts_the_from_set_itself_as_allowed() {
    let ok = model(&[("a", "a")], "kind: allow-only\nfrom: {layer: ui}\nto: {layer: domain}\n");
    assert!(codes(&ok, "E660").is_empty());
}

#[test]
fn a_layer_may_use_itself_and_layers_below_including_skipping() {
    // A (ui) depends on B (domain) and C (platform); B on C; all downward
    let ok = model(&[("b", "a"), ("c", "a"), ("c", "b")], LAYERS);
    assert!(codes(&ok, "E660").is_empty(), "{:?}", validate(&ok).findings);
}

#[test]
fn a_dependency_on_a_higher_layer_is_reported_with_both_layers() {
    // C.o -> ... the consumer depends on the producer: A.o -> C.i makes C (platform) depend on A (ui)
    let bad = model(&[("a", "c")], LAYERS);
    let e = codes(&bad, "E660");
    assert_eq!(e.len(), 1, "{e:?}");
    assert!(e[0].1.contains("P::C") && e[0].1.contains("P::A") && e[0].1.contains("platform") && e[0].1.contains("ui"), "{e:?}");
}

#[test]
fn a_component_without_a_listed_layer_is_unconstrained() {
    // D has no layer: it may depend on anything and anything may depend on it
    let ok = model(&[("a", "d"), ("d", "a"), ("d", "c")], LAYERS);
    assert!(codes(&ok, "E660").is_empty(), "{:?}", validate(&ok).findings);
}

#[test]
fn layer_is_also_a_selector_key() {
    let els = model(&[("c", "a")], "kind: forbid-dependency\nfrom: {layer: ui}\nto: {layer: platform}\n");
    assert_eq!(codes(&els, "E660").len(), 1, "{:?}", validate(&els).findings);
}

#[test]
fn malformed_layering_and_allow_only_rules_are_e662() {
    for rule in [
        "kind: layering\n",
        "kind: layering\nlayers: []\n",
        "kind: layering\nlayers: [ui, ui]\n",
        "kind: layering\nlayers: ui\n",
        "kind: allow-only\nfrom: {layer: ui}\n",
        "kind: allow-only\nto: {layer: ui}\n",
    ] {
        let els = model(&[], rule);
        assert_eq!(codes(&els, "E662").len(), 1, "{rule:?}: {:?}", validate(&els).findings);
    }
}

#[test]
fn layer_on_another_element_type_is_unrecognised() {
    let r = tempdir();
    write(&r, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(&r, "R.md", "---\ntype: Requirement\nid: REQ-LAY-001\nname: r\nstatus: draft\nreqDomain: software\nlayer: ui\n---\n\nThe system shall work.\n");
    let w = codes(&walk_model(&r).unwrap(), "W047");
    assert!(w.iter().any(|(_, m)| m.contains("'layer'")), "{w:?}");
}

// ── REQ-TRS-ARCH-007 ───────────────────────────────────────────────────────

const CHAIN: [(&str, &str); 3] = [("a", "b"), ("b", "c"), ("c", "d")]; // D depends on C depends on B depends on A

#[test]
fn a_chain_longer_than_max_depth_is_reported_with_the_chain() {
    let ok = model(&CHAIN, "kind: max-depth\nmaxDepth: 3\nfrom: {tag: chain}\nvia: [connections]\n");
    assert!(codes(&ok, "E663").is_empty(), "{:?}", validate(&ok).findings);
    let bad = model(&CHAIN, "kind: max-depth\nmaxDepth: 2\nfrom: {tag: chain}\nvia: [connections]\n");
    let e = codes(&bad, "E663");
    assert_eq!(e.len(), 1, "{e:?}");
    let m = &e[0].1;
    assert!(m.contains("AR-ARC-002") && m.contains("P::D") && m.contains('3') && m.contains("P::A"), "{m}");
}

#[test]
fn max_depth_without_from_covers_every_component() {
    let bad = model(&CHAIN, "kind: max-depth\nmaxDepth: 2\nvia: [connections]\n");
    assert_eq!(codes(&bad, "E663").len(), 1, "{:?}", validate(&bad).findings);
}

#[test]
fn a_dependency_cycle_terminates() {
    let els = model(&[("a", "b"), ("b", "c"), ("c", "d"), ("d", "a")], "kind: max-depth\nmaxDepth: 10\nvia: [connections]\n");
    assert!(codes(&els, "E663").is_empty(), "each component once on a chain: depth 3");
}

#[test]
fn max_depth_zero_forbids_any_dependency_and_a_bad_value_is_e662() {
    assert!(!codes(&model(&CHAIN, "kind: max-depth\nmaxDepth: 0\nvia: [connections]\n"), "E663").is_empty());
    for rule in ["kind: max-depth\n", "kind: max-depth\nmaxDepth: -1\n", "kind: max-depth\nmaxDepth: deep\n", "kind: max-depth\nmaxDepth: 1.5\n"] {
        let els = model(&[], rule);
        assert_eq!(codes(&els, "E662").len(), 1, "{rule:?}: {:?}", validate(&els).findings);
    }
}

#[test]
fn warning_severity_gives_w663() {
    let els = model(&CHAIN, "kind: max-depth\nmaxDepth: 1\nseverity: warning\nvia: [connections]\n");
    assert!(codes(&els, "E663").is_empty());
    assert!(!codes(&els, "W663").is_empty());
}

// ── REQ-TRS-ARCH-008 ───────────────────────────────────────────────────────

fn waive(w: &str) -> String {
    format!("{ALLOW}waivers:\n{w}")
}
const VIOLATING: [(&str, &str); 2] = [("b", "a"), ("c", "a")];

#[test]
fn a_current_waiver_suppresses_the_finding_for_its_element() {
    let rule = waive("  - {element: P::A, rationale: \"legacy wiring, ADR-LEG-001\", expires: \"2999-01-01\"}\n");
    let els = model(&VIOLATING, &rule);
    assert!(codes(&els, "E660").is_empty(), "{:?}", validate(&els).findings);
    assert!(codes(&els, "W664").is_empty(), "{:?}", validate(&els).findings);
}

#[test]
fn an_expired_waiver_lets_the_finding_back_and_is_w664() {
    let rule = waive("  - {element: P::A, rationale: \"temporary\", expires: \"2000-01-01\"}\n");
    let els = model(&VIOLATING, &rule);
    assert_eq!(codes(&els, "E660").len(), 1);
    let w = codes(&els, "W664");
    assert_eq!(w.len(), 1, "{w:?}");
    assert!(w[0].1.contains("P::A") && w[0].1.contains("expired"), "{w:?}");
}

#[test]
fn a_waiver_that_suppresses_nothing_is_w664() {
    let rule = waive("  - {element: P::C, rationale: \"not the dependent\", expires: \"2999-01-01\"}\n");
    let els = model(&VIOLATING, &rule);
    assert_eq!(codes(&els, "E660").len(), 1, "the dependent is A, not C");
    assert!(codes(&els, "W664").iter().any(|(_, m)| m.contains("nothing")), "{:?}", validate(&els).findings);
}

#[test]
fn a_waiver_applies_to_independence_through_the_shared_component() {
    // A and B both depend on C: shared = C
    let rule = "kind: independence\nfrom: {layer: ui}\nto: {layer: domain}\nwaivers:\n  - {element: P::C, rationale: \"common clock\", expires: \"2999-01-01\"}\n";
    let els = model(&[("c", "a"), ("c", "b")], rule);
    assert!(codes(&els, "E661").is_empty(), "{:?}", validate(&els).findings);
    let unwaived = model(&[("c", "a"), ("c", "b")], "kind: independence\nfrom: {layer: ui}\nto: {layer: domain}\n");
    assert_eq!(codes(&unwaived, "E661").len(), 1);
}

#[test]
fn a_malformed_waiver_is_e662() {
    for w in [
        "  - {element: P::A, expires: \"2999-01-01\"}\n",
        "  - {rationale: \"why\", expires: \"2999-01-01\"}\n",
        "  - {element: P::A, rationale: \"why\"}\n",
        "  - {element: P::A, rationale: \"why\", expires: \"soon\"}\n",
        "  - {element: P::A, rationale: \"why\", expires: \"2999-13-40\"}\n",
    ] {
        let els = model(&VIOLATING, &waive(w));
        assert_eq!(codes(&els, "E662").len(), 1, "{w:?}: {:?}", validate(&els).findings);
    }
}
