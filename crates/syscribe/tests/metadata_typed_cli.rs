//! TC-TRS-META-012, -013, -015 / REQ-TRS-META-012, -013, -015 (GH #375): what the typing buys
//! the user — defaults that show without being written, references that are traced, and
//! variant-parameter placeholders substituted under `--config`.
//!
//! Written before the implementation; every test here fails until it lands.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

fn model(def_features: &str, application: &str) -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let r = std::env::temp_dir().join(format!("syscribe-metacli-{}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst))).join("model");
    let w = |rel: &str, c: &str| {
        let p = r.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, c).unwrap();
    };
    w("_index.md", "---\ntype: Package\nname: Root\n---\n");
    w("B/_index.md", "---\ntype: Package\nname: B\n---\n");
    w("B/Safety.md", &format!("---\ntype: MetadataDef\nname: Safety\nfeatures:\n{def_features}---\n"));
    w("B/Other.md", "---\ntype: PartDef\nname: Other\n---\n");
    w("B/Item.md", &format!("---\ntype: PartDef\nname: Item\nmetadata:\n{application}---\n"));
    // A configuration binding Features::Grid.hz = 50, for the placeholder tests.
    w("Features/Grid.md", "---\ntype: FeatureDef\nid: FEAT-GRID\nname: Grid\ngroupKind: optional\nparameters:\n  - {name: hz, type: ScalarValues::Integer}\n---\n\nG.\n");
    w(
        "Configs/CONF-GRID-ALPHA.md",
        "---\nid: CONF-GRID-ALPHA\ntype: Configuration\nname: c\nstatus: approved\nfeatureModel: Features\nfeatures:\n  Features::Grid: true\nparameterBindings:\n  Features::Grid.hz: 50\n---\n\nC.\n",
    );
    r
}

fn run(root: &Path, args: &[&str]) -> (String, i32) {
    let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(root).args(args).output().unwrap();
    (format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr)), o.status.code().unwrap_or(-1))
}

/// The row of `list PartDef --json` for `B::Item`, optionally under a configuration.
fn item(root: &Path, config: Option<&str>) -> serde_json::Value {
    let mut a = vec!["list", "PartDef", "--json"];
    if let Some(c) = config {
        a.extend(["--config", c]);
    }
    let (o, c) = run(root, &a);
    assert_eq!(c, 0, "{o}");
    let v: serde_json::Value = serde_json::from_str(&o).unwrap_or_else(|e| panic!("{e}: {o}"));
    v.as_array().unwrap().iter().find(|r| r["qualifiedName"] == "B::Item").cloned().unwrap_or_else(|| panic!("B::Item in {o}"))
}

const LEVEL: &str = "  - name: level\n    typedBy: ScalarValues::Integer\n    default: 2\n";

// ── REQ-TRS-META-012 ───────────────────────────────────────────────────────

#[test]
fn a_default_is_shown_without_being_written_in_the_file() {
    let r = model(LEVEL, "  - B::Safety\n");
    let v = item(&r, None);
    let app = &v["metadata"][0];
    assert_eq!(app["def"], "B::Safety", "{v}");
    assert_eq!(app["values"]["level"], 2, "{v}");
    assert_eq!(app["defaulted"][0], "level", "the defaulted keys are marked: {v}");
    let (t, _) = run(&r, &["show", "B::Item"]);
    assert!(t.contains("level=2 (default)"), "text show marks the default: {t}");
    let file = std::fs::read_to_string(r.join("B/Item.md")).unwrap();
    assert!(!file.contains("level"), "the file is never rewritten: {file}");
}

#[test]
#[ignore = "needs `--where meta.…` (REQ-TRS-META-020, GH #376)"]
fn a_default_is_matched_by_list_where() {
    let r = model(LEVEL, "  - B::Safety\n");
    let (o, c) = run(&r, &["list", "PartDef", "--where", "meta.Safety.level=2"]);
    assert_eq!(c, 0, "{o}");
    assert!(o.contains("B::Item"), "{o}");
}

#[test]
fn an_explicit_value_wins_over_the_default() {
    let r = model(LEVEL, "  - type: B::Safety\n    level: 3\n");
    let v = item(&r, None);
    assert_eq!(v["metadata"][0]["values"]["level"], 3, "{v}");
    assert!(v["metadata"][0]["defaulted"].as_array().map_or(true, |a| a.is_empty()), "{v}");
}

// ── REQ-TRS-META-013 ───────────────────────────────────────────────────────

#[test]
fn a_tagged_value_reference_is_traced_by_refs_and_impact() {
    let r = model("  - name: reviewer\n    typedBy: PartDef\n", "  - type: B::Safety\n    reviewer: B::Other\n");
    let (o, c) = run(&r, &["refs", "B::Other"]);
    assert_eq!(c, 0, "{o}");
    assert!(o.contains("B::Item") && o.contains("Safety") && o.contains("reviewer"), "names holder, stereotype and key: {o}");
    let (o, _) = run(&r, &["impact", "B::Other"]);
    assert!(o.contains("B::Item"), "{o}");
}

// ── REQ-TRS-META-015 ───────────────────────────────────────────────────────

const HZ: &str = "  - name: hz\n    typedBy: ScalarValues::Integer\n    range: \"0..10\"\n";
const PLACEHOLDER: &str = "  - type: B::Safety\n    hz: \"{{Features::Grid.hz}}\"\n";

#[test]
fn a_placeholder_is_checked_as_the_substituted_value_under_config() {
    let r = model(HZ, PLACEHOLDER);
    let (o, c) = run(&r, &["validate", "--config", "CONF-GRID-ALPHA"]);
    assert_eq!(c, 1, "50 is outside 0..10: {o}");
    assert!(o.contains("E323") && o.contains("hz"), "{o}");
}

#[test]
fn a_placeholder_in_range_is_substituted_and_quiet() {
    let r = model("  - name: hz\n    typedBy: ScalarValues::Integer\n    range: \"0..100\"\n", PLACEHOLDER);
    let (o, c) = run(&r, &["validate", "--config", "CONF-GRID-ALPHA"]);
    assert_eq!(c, 0, "{o}");
    let v = item(&r, Some("CONF-GRID-ALPHA"));
    assert_eq!(v["metadata"][0]["values"]["hz"], 50, "{v}");
}

#[test]
fn without_a_configuration_the_placeholder_is_left_and_not_type_checked() {
    let r = model(HZ, PLACEHOLDER);
    let (o, c) = run(&r, &["validate"]);
    assert_eq!(c, 0, "no E322 for the unsubstituted text: {o}");
    assert!(!o.contains("E322"), "{o}");
}

#[test]
fn a_placeholder_naming_an_unknown_parameter_is_reported() {
    let r = model(HZ, "  - type: B::Safety\n    hz: \"{{Features::Grid.nope}}\"\n");
    let (o, c) = run(&r, &["validate", "--config", "CONF-GRID-ALPHA"]);
    assert_eq!(c, 1, "{o}");
    assert!(o.contains("nope"), "{o}");
}
