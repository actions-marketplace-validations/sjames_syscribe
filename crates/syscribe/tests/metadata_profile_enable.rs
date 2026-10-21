//! TC-TRS-META-031, -032, -033 / REQ-TRS-META-031..033 (GH #377): profiles enabled in
//! `.syscribe.toml`, the standard stereotype library, and automatic application.
//!
//! Written before the implementation; every test fails until it lands.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

struct M {
    root: PathBuf,
}

impl M {
    fn new(toml: &str) -> M {
        static N: AtomicU64 = AtomicU64::new(0);
        let root = std::env::temp_dir().join(format!("syscribe-metaprofen-{}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst))).join("model");
        let m = M { root };
        m.w("_index.md", "---\ntype: Package\nname: Root\n---\n");
        m.w("B/_index.md", "---\ntype: Package\nname: B\n---\n");
        if !toml.is_empty() {
            m.w(".syscribe.toml", toml);
        }
        m
    }
    fn w(&self, rel: &str, c: &str) -> &M {
        let p = self.root.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, c).unwrap();
        self
    }
    fn run(&self, args: &[&str]) -> (String, i32) {
        let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(&self.root).args(args).output().unwrap();
        (format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr)), o.status.code().unwrap_or(-1))
    }
    fn parts(&self, args: &[&str]) -> Vec<String> {
        let mut a = vec!["list", "PartDef", "--json"];
        a.extend_from_slice(args);
        let (o, c) = self.run(&a);
        assert_eq!(c, 0, "{o}");
        let v: serde_json::Value = serde_json::from_str(&o).unwrap_or_else(|e| panic!("{e}: {o}"));
        let mut n: Vec<String> = v.as_array().unwrap().iter().map(|r| r["name"].as_str().unwrap().to_string()).collect();
        n.sort();
        n
    }
}

const ENABLE: &str = "[metadata]\nprofiles = [\"Prof\"]\n";

/// A profile package `Prof` with a constrained `Regulatory`, and an element applying it
/// unverified.
fn profile_model(toml: &str) -> M {
    let m = M::new(toml);
    m.w("Prof/_index.md", "---\ntype: Package\nname: Prof\nprofile: true\nversion: \"1.2\"\n---\n\nA profile.\n");
    m.w("Prof/Regulatory.md", "---\ntype: MetadataDef\nname: Regulatory\nfeatures:\n  - name: status\n    typedBy: ScalarValues::String\nconstraints:\n  - 'status == \"verified\"'\n---\n");
    m.w("B/Item.md", "---\ntype: PartDef\nname: Item\nmetadata:\n  - type: Prof::Regulatory\n    status: unverified\n---\n\nI.\n");
    m
}

// ── REQ-TRS-META-031 ───────────────────────────────────────────────────────

#[test]
fn a_disabled_profile_leaves_the_model_unchanged() {
    let m = profile_model("");
    let (o, c) = m.run(&["validate"]);
    assert_eq!(c, 0, "{o}");
    assert!(!o.contains("E327"), "the profile's checks are dormant: {o}");
    assert!(!o.contains("W047"), "profile/version are recognised on a package: {o}");
    // Its definitions stay usable by reference.
    assert!(!o.contains("E317"), "{o}");
}

#[test]
fn an_enabled_profile_activates_its_checks() {
    let m = profile_model(ENABLE);
    let (o, c) = m.run(&["validate"]);
    assert_eq!(c, 1, "{o}");
    assert!(o.contains("E327") && o.contains("Regulatory") && o.contains("Item"), "{o}");
}

#[test]
fn a_definition_outside_any_profile_is_always_checked() {
    let m = profile_model("");
    m.w("B/Loose.md", "---\ntype: MetadataDef\nname: Loose\nfeatures:\n  - name: n\n    typedBy: ScalarValues::Integer\nconstraints:\n  - 'n > 0'\n---\n");
    m.w("B/Item2.md", "---\ntype: PartDef\nname: Item2\nmetadata:\n  - type: B::Loose\n    n: 0\n---\n\nI.\n");
    let (o, c) = m.run(&["validate"]);
    assert_eq!(c, 1, "{o}");
    assert!(o.contains("E327") && o.contains("Item2") && !o.contains("Regulatory"), "{o}");
}

#[test]
fn enabling_something_that_is_not_a_profile_is_w316() {
    let m = profile_model("[metadata]\nprofiles = [\"Prof\", \"B\", \"Nowhere\"]\n");
    let (o, _) = m.run(&["validate"]);
    let w: Vec<&str> = o.lines().filter(|l| l.contains("W316")).collect();
    assert_eq!(w.len(), 2, "B is not a profile: true package, Nowhere does not exist: {o}");
}

#[test]
fn profile_true_on_something_other_than_a_package_is_w047() {
    let m = M::new("");
    m.w("B/P.md", "---\ntype: PartDef\nname: P\nprofile: true\n---\n\nP.\n");
    let (o, _) = m.run(&["validate"]);
    assert!(o.lines().any(|l| l.contains("W047") && l.contains("profile")), "{o}");
}

// ── REQ-TRS-META-032 ──────────────────────────────────────────────────────

const STD: [&str; 7] = ["Safety", "Optional", "Regulatory", "External", "Environment", "Deployment", "Budget"];

#[test]
fn the_standard_library_resolves_when_it_is_used_without_e317() {
    let m = M::new("");
    m.w("B/Item.md", "---\ntype: PartDef\nname: Item\nmetadata:\n  - type: StdMetadata::Safety\n    level: 2\n  - StdMetadata::External\n---\n\nI.\n");
    let (o, c) = m.run(&["validate"]);
    assert_eq!(c, 0, "{o}");
    assert!(!o.contains("E317"), "{o}");
}

#[test]
fn the_library_is_typed() {
    let m = M::new("");
    m.w("B/Item.md", "---\ntype: PartDef\nname: Item\nmetadata:\n  - type: StdMetadata::Safety\n    level: 9\n  - type: StdMetadata::Regulatory\n    status: maybe\n  - type: StdMetadata::Deployment\n    kind: sandcastle\n---\n\nI.\n");
    let (o, c) = m.run(&["validate"]);
    assert_eq!(c, 1, "{o}");
    assert!(o.contains("E323") && o.lines().filter(|l| l.contains("E324")).count() == 2, "{o}");
}

#[test]
fn the_library_is_listed_when_enabled_and_absent_otherwise() {
    let off = M::new("");
    let (o, _) = off.run(&["list", "MetadataDef"]);
    assert!(!o.contains("StdMetadata"), "an unused library adds nothing to a model: {o}");
    let on = M::new("[metadata]\nstdlib = true\n");
    let (o, c) = on.run(&["list", "MetadataDef"]);
    assert_eq!(c, 0, "{o}");
    for d in STD {
        assert!(o.contains(&format!("StdMetadata::{d}")), "{d} in {o}");
    }
}

#[test]
fn a_project_definition_can_specialise_a_library_one() {
    let m = M::new("");
    m.w("B/Critical.md", "---\ntype: MetadataDef\nname: Critical\nsupertype: StdMetadata::Safety\n---\n");
    m.w("B/Item.md", "---\ntype: PartDef\nname: Item\nmetadata:\n  - type: B::Critical\n    level: 4\n---\n\nI.\n");
    let (o, c) = m.run(&["validate"]);
    assert_eq!(c, 0, "{o}");
    assert_eq!(m.parts(&["--metadata", "Safety"]), ["Item"]);
    assert_eq!(m.parts(&["--where", "meta.Safety.level>=4"]), ["Item"]);
}

#[test]
fn the_library_is_read_only() {
    let m = M::new("[metadata]\nstdlib = true\n");
    let (o, c) = m.run(&["set", "StdMetadata::Safety", "status=approved"]);
    assert_ne!(c, 0, "{o}");
    assert!(o.to_lowercase().contains("read-only") || o.to_lowercase().contains("built-in"), "{o}");
}

// ── REQ-TRS-META-033 ──────────────────────────────────────────────────────

const AUTO: &str = "---\ntype: Package\nname: Prof\nprofile: true\nautoApply:\n  - select:\n      type: PartDef\n      where: {domain: software, isDeploymentPackage: true}\n    apply: {type: StdMetadata::Deployment, kind: firmware}\n---\n\nA profile.\n";

fn auto_model(toml: &str) -> M {
    let m = M::new(toml);
    m.w("Prof/_index.md", AUTO);
    m.w("B/P1.md", "---\ntype: PartDef\nname: P1\ndomain: software\nisDeploymentPackage: true\n---\n\nP.\n");
    m.w("B/P2.md", "---\ntype: PartDef\nname: P2\ndomain: software\n---\n\nP.\n");
    m.w("B/P3.md", "---\ntype: PartDef\nname: P3\ndomain: software\nisDeploymentPackage: true\nmetadata:\n  - type: StdMetadata::Deployment\n    kind: process\n---\n\nP.\n");
    m
}

#[test]
fn a_derived_application_is_visible_to_queries() {
    let m = auto_model(ENABLE);
    assert_eq!(m.parts(&["--metadata", "Deployment"]), ["P1", "P3"]);
    assert_eq!(m.parts(&["--where", "meta.Deployment.kind=firmware"]), ["P1"]);
}

#[test]
fn a_disabled_profile_derives_nothing() {
    let m = auto_model("");
    assert_eq!(m.parts(&["--metadata", "Deployment"]), ["P3"]);
}

#[test]
fn the_derived_application_is_marked_and_never_written() {
    let m = auto_model(ENABLE);
    let (o, _) = m.run(&["list", "PartDef", "--json"]);
    let v: serde_json::Value = serde_json::from_str(&o).unwrap();
    let p1 = v.as_array().unwrap().iter().find(|x| x["name"] == "P1").unwrap();
    assert_eq!(p1["metadata"][0]["derived"], true, "{p1}");
    let (t, _) = m.run(&["show", "B::P1"]);
    assert!(t.contains("(derived)"), "{t}");
    let file = std::fs::read_to_string(m.root.join("B/P1.md")).unwrap();
    assert!(!file.contains("metadata"), "{file}");
}

#[test]
fn an_explicit_application_wins_over_the_derived_one() {
    let m = auto_model(ENABLE);
    let (o, _) = m.run(&["list", "PartDef", "--json"]);
    let v: serde_json::Value = serde_json::from_str(&o).unwrap();
    let p3 = v.as_array().unwrap().iter().find(|x| x["name"] == "P3").unwrap();
    assert_eq!(p3["metadata"].as_array().unwrap().len(), 1, "no duplicate: {p3}");
    assert_eq!(p3["metadata"][0]["values"]["kind"], "process", "{p3}");
    assert!(p3["metadata"][0].get("derived").is_none_or(|d| d == false), "{p3}");
}
