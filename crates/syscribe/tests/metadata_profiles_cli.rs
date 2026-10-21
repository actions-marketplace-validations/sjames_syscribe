//! TC-TRS-META-030 / REQ-TRS-META-030 (GH #377): queries follow specialisation.
//!
//! Written before the implementation.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

/// `Safety` (integrity, level default 2); `ASIL_D :> Safety`. `Plain` applies Safety, `Strict`
/// applies ASIL_D, `Bare` applies nothing.
fn model() -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let r = std::env::temp_dir().join(format!("syscribe-metaprofcli-{}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst))).join("model");
    let w = |rel: &str, c: &str| {
        let p = r.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, c).unwrap();
    };
    w("_index.md", "---\ntype: Package\nname: Root\n---\n");
    w("B/_index.md", "---\ntype: Package\nname: B\n---\n");
    w("B/Safety.md", "---\ntype: MetadataDef\nname: Safety\nfeatures:\n  - name: integrity\n    typedBy: ScalarValues::Integer\n  - name: level\n    typedBy: ScalarValues::Integer\n    default: 2\n---\n");
    w("B/ASIL_D.md", "---\ntype: MetadataDef\nname: ASIL_D\nsupertype: B::Safety\nfeatures:\n  - name: rationale\n    typedBy: ScalarValues::String\n---\n");
    w("B/Plain.md", "---\ntype: PartDef\nname: Plain\nmetadata:\n  - type: B::Safety\n    integrity: 1\n---\n");
    w("B/Strict.md", "---\ntype: PartDef\nname: Strict\nmetadata:\n  - type: B::ASIL_D\n    integrity: 4\n---\n");
    w("B/Bare.md", "---\ntype: PartDef\nname: Bare\n---\n");
    r
}

fn run(root: &Path, args: &[&str]) -> (String, i32) {
    let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(root).args(args).output().unwrap();
    (format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr)), o.status.code().unwrap_or(-1))
}

fn parts(root: &Path, args: &[&str]) -> Vec<String> {
    let mut a = vec!["list", "PartDef", "--json"];
    a.extend_from_slice(args);
    let (o, c) = run(root, &a);
    assert_eq!(c, 0, "{o}");
    let v: serde_json::Value = serde_json::from_str(&o).unwrap_or_else(|e| panic!("{e}: {o}"));
    let mut n: Vec<String> = v.as_array().unwrap().iter().map(|r| r["name"].as_str().unwrap().to_string()).collect();
    n.sort();
    n
}

#[test]
fn listing_the_supertype_finds_sub_definition_applications() {
    let r = model();
    assert_eq!(parts(&r, &["--metadata", "Safety"]), ["Plain", "Strict"]);
}

#[test]
fn listing_the_sub_definition_does_not_find_plain_supertype_applications() {
    let r = model();
    assert_eq!(parts(&r, &["--metadata", "ASIL_D"]), ["Strict"]);
}

#[test]
fn negation_follows_specialisation() {
    let r = model();
    assert_eq!(parts(&r, &["--metadata", "!Safety"]), ["Bare"]);
}

#[test]
fn a_where_on_the_supertype_sees_inherited_values_of_sub_definition_applications() {
    let r = model();
    assert_eq!(parts(&r, &["--where", "meta.Safety.integrity>=2"]), ["Strict"]);
    assert_eq!(parts(&r, &["--where", "meta.Safety"]), ["Plain", "Strict"]);
    // The inherited default applies to the sub-definition application too.
    assert_eq!(parts(&r, &["--where", "meta.Safety.level=2"]), ["Plain", "Strict"]);
    assert_eq!(parts(&r, &["--where", "meta.ASIL_D.integrity>=2"]), ["Strict"]);
}

#[test]
fn json_shows_the_inherited_default_on_a_sub_definition_application() {
    let r = model();
    let (o, _) = run(&r, &["list", "PartDef", "--json"]);
    let v: serde_json::Value = serde_json::from_str(&o).unwrap();
    let strict = v.as_array().unwrap().iter().find(|x| x["name"] == "Strict").unwrap();
    let app = &strict["metadata"][0];
    assert_eq!(app["def"], "B::ASIL_D");
    assert_eq!(app["values"]["level"], 2, "{app}");
    assert_eq!(app["defaulted"][0], "level", "{app}");
}
