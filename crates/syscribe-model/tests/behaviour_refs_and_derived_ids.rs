//! GH #224: a type used only as a transition `accept` payload or an
//! `entryAction`/`effect` is "used" (no `W007`). GH #227: a `featureTree:` id
//! derived from a long name is cut to a valid FEAT segment (no `E006`).

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use syscribe_model::validator;
use syscribe_model::walker::walk_model;

fn tempdir() -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let p = std::env::temp_dir().join(format!(
        "syscribe-behaviour-refs-{}-{}",
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

fn findings(root: &Path) -> Vec<validator::Finding> {
    let elements = walk_model(root).expect("walk");
    validator::validate(&elements).findings
}

#[test]
fn behaviour_only_references_are_uses() {
    let root = tempdir();
    write(&root, "Events/IgnitionOn.md", "---\ntype: ItemDef\nname: IgnitionOn\n---\nAn event.\n");
    write(&root, "Behavior/Boot.md", "---\ntype: ActionDef\nname: Boot\n---\nBoot.\n");
    write(&root, "Behavior/Cleanup.md", "---\ntype: ActionDef\nname: Cleanup\n---\nCleanup.\n");
    write(
        &root,
        "Behavior/Modes.md",
        "---\ntype: StateDef\nname: Modes\nsubStates:\n  - name: Off\n    isInitial: true\n    transitions:\n      - target: On\n        accept: Events::IgnitionOn\n        effect: Behavior::Cleanup\n  - name: On\n    entryAction: Behavior::Boot\n    transitions:\n      - target: Off\n---\nModes.\n",
    );
    let f = findings(&root);
    let unused: Vec<_> = f.iter().filter(|x| x.code == "W007").map(|x| x.message.clone()).collect();
    assert!(!unused.iter().any(|m| m.contains("IgnitionOn")), "accept payload is a use: {unused:?}");
    assert!(!unused.iter().any(|m| m.contains("Boot")), "entryAction is a use: {unused:?}");
    assert!(!unused.iter().any(|m| m.contains("Cleanup")), "effect is a use: {unused:?}");
}

#[test]
fn long_derived_feature_id_is_valid() {
    let root = tempdir();
    write(
        &root,
        "Features/_index.md",
        "---\ntype: FeatureModel\nname: Features\nfeatureTree:\n  - {name: Platform, groupKind: or}\n  - {name: Platform.HighPerformance}\n  - {name: Platform.ClusterDisplay}\n---\n",
    );
    let f = findings(&root);
    assert!(!f.iter().any(|x| x.code == "E006"), "derived ids fit the FEAT pattern: {:?}", f.iter().filter(|x| x.code == "E006").collect::<Vec<_>>());
}
