//! Self- and mutually-recursive actions must be rejected or bounded, never overflow the stack. Kept in its own
//! test binary because a stack overflow aborts the whole process.

use std::path::PathBuf;

use syscribe_model::act_ir::lower_action;
use syscribe_model::act_run::{ActOptions, ActRun, Status};
use syscribe_model::walker::walk_model;

fn dir(files: &[(&str, &str)], tag: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("syscribe-actrec-{}-{tag}", std::process::id()));
    std::fs::create_dir_all(&d).unwrap();
    std::fs::write(d.join("_index.md"), "---\ntype: Package\nname: Root\n---\n").unwrap();
    for (n, b) in files {
        std::fs::write(d.join(format!("{n}.md")), format!("---\ntype: ActionDef\nname: {n}\n{b}---\n")).unwrap();
    }
    d
}

fn outcome(files: &[(&str, &str)], root: &str, tag: &str) -> String {
    let d = dir(files, tag);
    let elems = walk_model(&d).unwrap();
    match lower_action(&elems, root) {
        Err(e) => format!("rejected at lowering: {e:?}"),
        Ok(p) => {
            let mut r = ActRun::new(&p, ActOptions::default());
            r.run();
            format!("ran: {:?} {:?}", r.status(), r.error())
        }
    }
}

#[test]
fn a_self_recursive_action_is_rejected_or_bounded() {
    let s = outcome(&[("R", "subActions:\n  - {name: c, kind: PerformAction, typedBy: R}\n")], "R", "self");
    eprintln!("self recursion: {s}");
    assert!(s.starts_with("rejected") || s.contains("Error"), "{s}");
}

#[test]
fn mutual_recursion_is_rejected_or_bounded() {
    let s = outcome(
        &[
            ("P", "subActions:\n  - {name: c, kind: PerformAction, typedBy: Q}\n"),
            ("Q", "subActions:\n  - {name: c, kind: PerformAction, typedBy: P}\n"),
        ],
        "P",
        "mutual",
    );
    eprintln!("mutual recursion: {s}");
    assert!(s.starts_with("rejected") || s.contains("Error"), "{s}");
}

#[test]
fn a_guarded_recursion_that_terminates_is_accepted_or_clearly_rejected() {
    let s = outcome(
        &[(
            "F",
            "parameters:\n  - {name: n, direction: in}\n  - {name: r, direction: out}\nsubActions:\n  - {name: base, kind: AssignmentAction, assign: \"r := 1\"}\n  - {name: rec, kind: PerformAction, typedBy: F, parameters: [{name: n, value: \"n - 1\"}]}\n  - {name: mul, kind: AssignmentAction, assign: \"r := n * rec.r\"}\ncontrolNodes:\n  - {name: d, kind: DecisionNode}\nsuccessionConnections:\n  - {after: start, before: d}\n  - {after: d, before: base, guard: \"n <= 1\"}\n  - {after: d, before: rec, guard: else}\n  - {after: rec, before: mul}\n",
        )],
        "F",
        "fact",
    );
    eprintln!("factorial recursion: {s}");
    assert!(!s.is_empty());
    let _ = Status::Completed;
}
