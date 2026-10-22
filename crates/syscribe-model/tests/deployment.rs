//! TC-TRS-DEPL-001..003 / REQ-TRS-DEPL-001..003 (GH #370, slice one): `deploymentKind:` on parts, nesting order,
//! a component deployed once unless replicated, and integrity of host versus hosted.
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
    let p = std::env::temp_dir().join(format!("syscribe-depl-{}-{}", std::process::id(), N.fetch_add(1, Ordering::Relaxed)));
    std::fs::create_dir_all(&p).unwrap();
    p
}

/// Parts from `(path, frontmatter body)` under a root package.
fn model(parts: &[(&str, &str)]) -> Vec<RawElement> {
    let r = tempdir();
    write(&r, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    for p in ["Hw", "Sw"] {
        write(&r, &format!("{p}/_index.md"), &format!("---\ntype: Package\nname: {p}\n---\n"));
    }
    for (path, body) in parts {
        let name = path.rsplit('/').next().unwrap().trim_end_matches(".md");
        write(&r, path, &format!("---\ntype: PartDef\nname: {name}\n{body}---\n\n{name}.\n"));
    }
    walk_model(&r).unwrap()
}

fn codes(els: &[RawElement], code: &str) -> Vec<(String, String)> {
    validate(els).findings.iter().filter(|f| f.code == code).map(|f| (f.file.clone(), f.message.clone())).collect()
}

fn node(path: &'static str, kind: &'static str) -> (&'static str, String) {
    (path, format!("domain: hardware\ndeploymentKind: {kind}\n"))
}

fn run(nodes: &[(&str, String)], sw: &[(&str, &str)]) -> Vec<RawElement> {
    let mut parts: Vec<(&str, String)> = nodes.to_vec();
    parts.extend(sw.iter().map(|(p, b)| (*p, b.to_string())));
    let borrowed: Vec<(&str, &str)> = parts.iter().map(|(p, b)| (*p, b.as_str())).collect();
    model(&borrowed)
}

// ── REQ-TRS-DEPL-001 ───────────────────────────────────────────────────────

#[test]
fn every_kind_is_accepted_and_an_unknown_one_is_e730() {
    for k in ["soc", "core", "partition", "container", "process", "thread"] {
        let els = run(&[node("Hw/N.md", k)], &[]);
        assert!(codes(&els, "E730").is_empty(), "{k}");
    }
    let bad = run(&[node("Hw/N.md", "cluster")], &[]);
    let e = codes(&bad, "E730");
    assert_eq!(e.len(), 1, "{e:?}");
    assert!(e[0].1.contains("cluster") && e[0].0.ends_with("N.md"), "{e:?}");
}

#[test]
fn nodes_nest_outermost_first() {
    let nodes = [
        ("Hw/Soc.md", "domain: hardware\ndeploymentKind: soc\nfeatures:\n  - {name: p, typedBy: Hw::Part1}\n".to_string()),
        ("Hw/Part1.md", "domain: hardware\ndeploymentKind: partition\nfeatures:\n  - {name: pr, typedBy: Hw::Proc}\n".to_string()),
        node("Hw/Proc.md", "process"),
    ];
    assert!(codes(&run(&nodes, &[]), "E731").is_empty());
}

#[test]
fn a_node_inside_the_same_or_a_later_kind_is_e731() {
    let inverted = [
        ("Hw/Proc.md", "domain: hardware\ndeploymentKind: process\nfeatures:\n  - {name: p, typedBy: Hw::Part1}\n".to_string()),
        node("Hw/Part1.md", "partition"),
    ];
    let e = codes(&run(&inverted, &[]), "E731");
    assert_eq!(e.len(), 1, "{e:?}");
    assert!(e[0].1.contains("Hw::Proc") && e[0].1.contains("Hw::Part1"), "{e:?}");
    let same = [
        ("Hw/A.md", "domain: hardware\ndeploymentKind: core\nfeatures:\n  - {name: b, typedBy: Hw::B}\n".to_string()),
        node("Hw/B.md", "core"),
    ];
    assert_eq!(codes(&run(&same, &[]), "E731").len(), 1);
}

#[test]
fn a_part_without_a_kind_is_not_constrained_and_deployment_kind_elsewhere_is_w047() {
    let nodes = [("Hw/A.md", "domain: hardware\nfeatures:\n  - {name: b, typedBy: Hw::B}\n".to_string()), node("Hw/B.md", "soc")];
    assert!(codes(&run(&nodes, &[]), "E731").is_empty());
    let r = tempdir();
    write(&r, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(&r, "R.md", "---\ntype: Requirement\nid: REQ-DEP-001\nname: r\nstatus: draft\nreqDomain: software\ndeploymentKind: soc\nreplicated: true\n---\n\nThe system shall work.\n");
    let w = codes(&walk_model(&r).unwrap(), "W047");
    for f in ["deploymentKind", "replicated"] {
        assert!(w.iter().any(|(_, m)| m.contains(&format!("'{f}'"))), "{f}: {w:?}");
    }
}

// ── REQ-TRS-DEPL-002 ───────────────────────────────────────────────────────

#[test]
fn a_component_on_two_nodes_is_e732_unless_replicated() {
    let nodes = [node("Hw/P1.md", "partition"), node("Hw/P2.md", "partition")];
    let twice = run(&nodes, &[("Sw/App.md", "domain: software\nallocatedTo: [Hw::P1, Hw::P2]\n")]);
    let e = codes(&twice, "E732");
    assert_eq!(e.len(), 1, "{e:?}");
    assert!(e[0].1.contains("Sw::App") && e[0].1.contains("Hw::P1") && e[0].1.contains("Hw::P2"), "{e:?}");
    let ok = run(&nodes, &[("Sw/App.md", "domain: software\nreplicated: true\nallocatedTo: [Hw::P1, Hw::P2]\n")]);
    assert!(codes(&ok, "E732").is_empty());
    let once = run(&nodes, &[("Sw/App.md", "domain: software\nallocatedTo: [Hw::P1]\n")]);
    assert!(codes(&once, "E732").is_empty());
}

#[test]
fn only_nodes_with_a_kind_are_counted() {
    let nodes = [node("Hw/P1.md", "partition"), ("Hw/Board.md", "domain: hardware\n".to_string())];
    let els = run(&nodes, &[("Sw/App.md", "domain: software\nallocatedTo: [Hw::P1, Hw::Board]\n")]);
    assert!(codes(&els, "E732").is_empty(), "{:?}", validate(&els).findings);
}

// ── REQ-TRS-DEPL-003 ───────────────────────────────────────────────────────

#[test]
fn a_node_with_a_lower_integrity_than_its_component_is_e733() {
    let low = [("Hw/P1.md", "domain: hardware\ndeploymentKind: partition\nasilLevel: B\n".to_string())];
    let bad = run(&low, &[("Sw/App.md", "domain: software\nasilLevel: D\nallocatedTo: [Hw::P1]\n")]);
    let e = codes(&bad, "E733");
    assert_eq!(e.len(), 1, "{e:?}");
    assert!(e[0].1.contains("Sw::App") && e[0].1.contains("Hw::P1") && e[0].1.contains('B') && e[0].1.contains('D'), "{e:?}");
    for (host, guest) in [("D", "B"), ("B", "B"), ("C", "QM")] {
        let n = [("Hw/P1.md", format!("domain: hardware\ndeploymentKind: partition\nasilLevel: {host}\n"))];
        let ok = run(&n, &[("Sw/App.md", &format!("domain: software\nasilLevel: {guest}\nallocatedTo: [Hw::P1]\n"))]);
        assert!(codes(&ok, "E733").is_empty(), "{host} hosts {guest}");
    }
}

#[test]
fn sil_is_compared_by_number_and_undeclared_levels_are_not_checked() {
    let n = [("Hw/P1.md", "domain: hardware\ndeploymentKind: partition\nsilLevel: 2\n".to_string())];
    assert_eq!(codes(&run(&n, &[("Sw/App.md", "domain: software\nsilLevel: 3\nallocatedTo: [Hw::P1]\n")]), "E733").len(), 1);
    assert!(codes(&run(&n, &[("Sw/App.md", "domain: software\nsilLevel: 2\nallocatedTo: [Hw::P1]\n")]), "E733").is_empty());
    let none = [node("Hw/P1.md", "partition")];
    assert!(codes(&run(&none, &[("Sw/App.md", "domain: software\nasilLevel: D\nallocatedTo: [Hw::P1]\n")]), "E733").is_empty(), "node declares none");
    assert!(codes(&run(&n, &[("Sw/App.md", "domain: software\nallocatedTo: [Hw::P1]\n")]), "E733").is_empty(), "component declares none");
}
