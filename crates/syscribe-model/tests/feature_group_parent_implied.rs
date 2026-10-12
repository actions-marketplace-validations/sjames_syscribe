//! GH #228: selecting a feature selects its group parents. A `Configuration`
//! that selects only children does not make a binding on the parent `E203`,
//! and does not leave the parent an orphan (`W024`).

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use syscribe_model::feature_model;
use syscribe_model::validator;
use syscribe_model::walker::walk_model;

fn tempdir() -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let p = std::env::temp_dir().join(format!(
        "syscribe-group-parent-{}-{}",
        std::process::id(),
        N.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&p).unwrap();
    p
}

fn write(root: &Path, rel: &str, content: &str) {
    let path = root.join(rel);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, content).unwrap();
}

fn model(eco: bool, perf: bool) -> PathBuf {
    let root = tempdir();
    write(&root, "_index.md", "---\ntype: Package\nname: M\n---\n");
    write(
        &root,
        "FM.md",
        "---\ntype: FeatureModel\nname: FM\nfeatureTree:\n  - name: Platform\n    id: FEAT-PLATFORM\n    groupKind: or\n    parameters:\n      - {name: clock, type: Integer, default: 100}\n  - {name: Platform.Eco, id: FEAT-ECO}\n  - {name: Platform.Perf, id: FEAT-PERF}\n---\n",
    );
    write(
        &root,
        "CONF-AAA.md",
        &format!("---\nid: CONF-AAA\ntype: Configuration\nname: A\nstatus: approved\nfeatureModel: FM\nfeatures:\n  FEAT-ECO: {eco}\n  FEAT-PERF: {perf}\nparameterBindings:\n  FM::Platform.clock: 200\n---\n"),
    );
    root
}

#[test]
fn selected_child_implies_parent_for_binding_and_orphan() {
    let elements = walk_model(&model(true, false)).unwrap();
    let v = validator::validate(&elements);
    assert!(!v.findings.iter().any(|f| f.code == "E203"), "{:#?}", v.findings);
    let fc = feature_model::check_feature_model(&elements);
    assert!(
        !fc.iter().any(|f| f.code == "W024" && f.message.contains("FM::Platform'")),
        "{:#?}",
        fc
    );
}

#[test]
fn no_selected_descendant_still_raises_e203() {
    let elements = walk_model(&model(false, false)).unwrap();
    let v = validator::validate(&elements);
    assert!(v.findings.iter().any(|f| f.code == "E203"), "{:#?}", v.findings);
}
