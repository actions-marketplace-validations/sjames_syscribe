//! TC-TRS-META-053 / REQ-TRS-META-053 (GH #379): `[[gates.coverage]]`, a coverage threshold for the
//! requirements a stereotype selector chooses, enforced by `validate` (exit 2).
//!
//! Written before the implementation.

use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

struct M {
    root: PathBuf,
}

impl M {
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
    fn tc(&self, id: &str, verifies: &str) {
        self.w(&format!("T/{id}.md"), &format!("---\ntype: TestCase\nid: {id}\nname: t\nstatus: active\ntestLevel: L2\nverifies: [{verifies}]\n---\n\n```gherkin\nFeature: f\n\n  Scenario: s\n    Given a\n    Then b\n```\n"));
    }
}

/// `REQ-GC-001` Safety(3) with an active test (complete); `-002` Safety(3) without; `-003` Safety(1) without;
/// `-004` a draft Safety(3) without; `-005` ASIL_D(4), a Safety, without; `-006` no stereotype, without.
fn model(gates: &str) -> M {
    static N: AtomicU64 = AtomicU64::new(0);
    let root = std::env::temp_dir().join(format!("syscribe-covgate-{}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst))).join("model");
    let m = M { root };
    m.w("_index.md", "---\ntype: Package\nname: Root\n---\n");
    m.w("B/_index.md", "---\ntype: Package\nname: B\n---\n");
    m.w("B/Safety.md", "---\ntype: MetadataDef\nname: Safety\nfeatures:\n  - name: integrity\n    typedBy: ScalarValues::Integer\n---\n");
    m.w("B/ASIL_D.md", "---\ntype: MetadataDef\nname: ASIL_D\nsupertype: B::Safety\n---\n");
    let req = |id: &str, status: &str, meta: &str| m.w(&format!("R/{id}.md"), &format!("---\ntype: Requirement\nid: {id}\nname: r\nstatus: {status}\nreqDomain: software\n{meta}---\n\nThe system shall work.\n"));
    req("REQ-GC-001", "approved", "metadata:\n  - type: B::Safety\n    integrity: 3\n");
    req("REQ-GC-002", "approved", "metadata:\n  - type: B::Safety\n    integrity: 3\n");
    req("REQ-GC-003", "approved", "metadata:\n  - type: B::Safety\n    integrity: 1\n");
    req("REQ-GC-004", "draft", "metadata:\n  - type: B::Safety\n    integrity: 3\n");
    m.tc("TC-GC-001", "REQ-GC-001");
    m.w(".syscribe.toml", gates);
    m
}

const SAFE: &str = "[[gates.coverage]]\nselect = \"Safety where integrity >= 2\"\n";

#[test]
fn a_selected_set_below_the_threshold_trips_the_gate_and_names_the_incomplete_requirements() {
    let (o, c) = model(SAFE).run(&["validate"]);
    assert_eq!(c, 2, "a tripped gate is exit 2: {o}");
    assert!(o.contains("1/2") && o.contains("50%") && o.contains("100%"), "{o}");
    assert!(o.contains("REQ-GC-002"), "the incomplete one: {o}");
    let gate: Vec<&str> = o.lines().filter(|l| l.contains("coverage gate")).collect();
    assert_eq!(gate.len(), 1, "{o}");
    for not_listed in ["REQ-GC-001", "REQ-GC-003", "REQ-GC-004"] {
        assert!(!gate[0].contains(not_listed), "{not_listed} is complete, below the selector, or a draft: {}", gate[0]);
    }
}

#[test]
fn the_default_threshold_is_every_selected_requirement() {
    let m = model(SAFE);
    m.tc("TC-GC-002", "REQ-GC-002");
    let (o, c) = m.run(&["validate"]);
    assert_eq!(c, 0, "both selected requirements now have an active test: {o}");
    assert!(!o.contains("coverage gate"), "{o}");
}

#[test]
fn min_complete_sets_the_percentage() {
    let ok = model(&format!("{SAFE}min_complete = 50\n"));
    assert_eq!(ok.run(&["validate"]).1, 0, "1/2 is 50%");
    let no = model(&format!("{SAFE}min_complete = 51\n"));
    assert_eq!(no.run(&["validate"]).1, 2);
}

#[test]
fn a_specialisation_is_selected_by_its_supertype_and_drafts_are_not_counted() {
    let m = model(&format!("{SAFE}min_complete = 50\n"));
    m.w("R/REQ-GC-005.md", "---\ntype: Requirement\nid: REQ-GC-005\nname: r\nstatus: approved\nreqDomain: software\nmetadata:\n  - type: B::ASIL_D\n    integrity: 4\n---\n\nThe system shall work.\n");
    let (o, c) = m.run(&["validate"]);
    assert_eq!(c, 2, "1/3 complete once the ASIL_D requirement counts: {o}");
    assert!(o.contains("1/3") && o.contains("REQ-GC-005"), "{o}");
}

#[test]
fn a_gate_with_no_stereotype_condition_counts_all_that_apply_it() {
    let m = model("[[gates.coverage]]\nselect = \"Safety\"\nmin_complete = 30\n");
    let (o, c) = m.run(&["validate"]);
    assert_eq!(c, 0, "1/3 (the draft is not counted) is 33%: {o}");
}

#[test]
fn several_gates_are_each_reported() {
    let m = model(&format!("{SAFE}\n[[gates.coverage]]\nselect = \"Safety where integrity < 2\"\nmin_complete = 100\n"));
    let (o, c) = m.run(&["validate"]);
    assert_eq!(c, 2, "{o}");
    assert_eq!(o.lines().filter(|l| l.contains("coverage gate")).count(), 2, "{o}");
}

#[test]
fn malformed_or_empty_gates_are_usage_errors() {
    for bad in [
        "[[gates.coverage]]\nselect = \"Safety where integrity >=\"\n",
        "[[gates.coverage]]\nselect = \"Nonesuch\"\n",
        "[[gates.coverage]]\nselect = \"Safety\"\nmin_complete = 150\n",
        "[[gates.coverage]]\nselect = \"Safety\"\nmin_complete = -1\n",
        "[[gates.coverage]]\nselect = \"Safety\"\nbogus = 1\n",
        "[[gates.coverage]]\nmin_complete = 50\n",
        "[[gates.coverage]]\nselect = \"Safety where integrity > 99\"\n",
    ] {
        let (o, c) = model(bad).run(&["validate"]);
        assert_eq!(c, 1, "{bad:?}: {o}");
    }
}

#[test]
fn validate_with_errors_still_exits_1_and_without_a_gate_nothing_changes() {
    let m = model("");
    assert_eq!(m.run(&["validate"]).1, 0);
    let broken = model(SAFE);
    broken.w("R/REQ-GC-009.md", "---\ntype: Requirement\nid: REQ-GC-009\nname: r\nstatus: approved\nreqDomain: software\nderivedFrom: [REQ-NOPE-001]\n---\n\nThe system shall work.\n");
    assert_eq!(broken.run(&["validate"]).1, 1, "errors dominate a tripped gate");
}
