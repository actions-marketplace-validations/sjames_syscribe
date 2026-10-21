//! TC-TRS-BUD-004..006 / REQ-TRS-BUD-004..006 (GH #368, CPU half): timing fields on software parts, `wcet` against
//! `deadline`, and CPU utilisation of what is allocated to a processor.
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
    let p = std::env::temp_dir().join(format!("syscribe-cpu-{}-{}", std::process::id(), N.fetch_add(1, Ordering::Relaxed)));
    std::fs::create_dir_all(&p).unwrap();
    p
}

/// `Hw::Cpu` (hardware, extra frontmatter `hw`) and software tasks `Sw::<name>` allocated to it, each with extra lines.
fn model(hw: &str, tasks: &[(&str, &str)], extra: &[(&str, &str)]) -> Vec<RawElement> {
    let r = tempdir();
    write(&r, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    for p in ["Hw", "Sw"] {
        write(&r, &format!("{p}/_index.md"), &format!("---\ntype: Package\nname: {p}\n---\n"));
    }
    write(&r, "Hw/Cpu.md", &format!("---\ntype: PartDef\nname: Cpu\ndomain: hardware\n{hw}---\n\nCpu.\n"));
    for (name, fm) in tasks {
        write(&r, &format!("Sw/{name}.md"), &format!("---\ntype: PartDef\nname: {name}\ndomain: software\nallocatedTo: [Hw::Cpu]\n{fm}---\n\n{name}.\n"));
    }
    for (p, c) in extra {
        write(&r, p, c);
    }
    walk_model(&r).unwrap()
}

fn codes(els: &[RawElement], code: &str) -> Vec<(String, String)> {
    validate(els).findings.iter().filter(|f| f.code == code).map(|f| (f.file.clone(), f.message.clone())).collect()
}

fn task(wcet: &str, period: &str) -> String {
    format!("wcet: \"{wcet}\"\nperiod: \"{period}\"\n")
}

// ── REQ-TRS-BUD-006: utilisation ───────────────────────────────────────────

#[test]
fn utilisation_within_cpu_margin_passes_and_above_it_names_the_tasks() {
    // 40 % + 29 % = 69 % of a 70 % limit
    let ok = model("cpuMargin: 70\n", &[("Fast", &task("4 ms", "10 ms")), ("Slow", &task("29 ms", "100 ms"))], &[]);
    assert!(codes(&ok, "E692").is_empty(), "{:?}", validate(&ok).findings);
    let over = model("cpuMargin: 70\n", &[("Fast", &task("4 ms", "10 ms")), ("Slow", &task("31 ms", "100 ms"))], &[]);
    let e = codes(&over, "E692");
    assert_eq!(e.len(), 1, "{e:?}");
    assert!(e[0].0.ends_with("Cpu.md"), "{e:?}");
    let m = &e[0].1;
    assert!(m.contains("Hw::Cpu") && m.contains("71") && m.contains("70"), "{m}");
    assert!(m.find("Sw::Fast").unwrap() < m.find("Sw::Slow").unwrap(), "largest first: {m}");
}

#[test]
fn utilisation_equal_to_the_limit_passes() {
    let els = model("cpuMargin: 70\n", &[("A", &task("7 ms", "10 ms"))], &[]);
    assert!(codes(&els, "E692").is_empty());
}

#[test]
fn edf_is_the_default_bound_of_one() {
    let ok = model("scheduling: edf\n", &[("A", &task("6 ms", "10 ms")), ("B", &task("4 ms", "10 ms"))], &[]);
    assert!(codes(&ok, "E692").is_empty(), "100 %");
    let over = model("scheduling: edf\n", &[("A", &task("6 ms", "10 ms")), ("B", &task("5 ms", "10 ms"))], &[]);
    assert_eq!(codes(&over, "E692").len(), 1, "110 %");
}

#[test]
fn rate_monotonic_uses_the_liu_layland_bound() {
    // two tasks: 2 (sqrt 2 - 1) = 0.8284; 41 % + 41 % = 82 % passes, 42 % + 42 % = 84 % fails
    let ok = model("scheduling: rate-monotonic\n", &[("A", &task("41 ms", "100 ms")), ("B", &task("41 ms", "100 ms"))], &[]);
    assert!(codes(&ok, "E692").is_empty(), "{:?}", validate(&ok).findings);
    let over = model("scheduling: rate-monotonic\n", &[("A", &task("42 ms", "100 ms")), ("B", &task("42 ms", "100 ms"))], &[]);
    let e = codes(&over, "E692");
    assert_eq!(e.len(), 1, "{e:?}");
    assert!(e[0].1.contains("rate-monotonic") && e[0].1.contains("82.8"), "the bound is named: {e:?}");
}

#[test]
fn the_margin_scales_the_bound() {
    // rate-monotonic, two tasks: 0.8284 x 50 % = 41.4 %; 42 % fails
    let els = model("scheduling: rate-monotonic\ncpuMargin: 50\n", &[("A", &task("21 ms", "100 ms")), ("B", &task("21 ms", "100 ms"))], &[]);
    assert_eq!(codes(&els, "E692").len(), 1);
}

#[test]
fn contained_tasks_count_once_whether_or_not_they_are_also_allocated() {
    let extra = [(
        "Sw/Bundle.md",
        "---\ntype: PartDef\nname: Bundle\ndomain: software\nallocatedTo: [Hw::Cpu]\nfeatures:\n  - {name: a, typedBy: Sw::A}\n  - {name: b, typedBy: Sw::B}\n---\n\nB.\n",
    )];
    // A and B are allocated directly and contained in Bundle: 35 % + 35 % = 70 %, once
    let ok = model("cpuMargin: 70\n", &[("A", &task("35 ms", "100 ms")), ("B", &task("35 ms", "100 ms"))], &extra);
    assert!(codes(&ok, "E692").is_empty(), "{:?}", validate(&ok).findings);
    let over = model("cpuMargin: 70\n", &[("A", &task("36 ms", "100 ms")), ("B", &task("35 ms", "100 ms"))], &extra);
    assert_eq!(codes(&over, "E692").len(), 1);
}

#[test]
fn a_part_that_declares_no_cpu_budget_is_not_checked_and_untimed_software_is_not_a_task() {
    let none = model("", &[("A", &task("99 ms", "100 ms")), ("B", &task("99 ms", "100 ms"))], &[]);
    assert!(codes(&none, "E692").is_empty(), "no cpuMargin, no scheduling");
    let untimed = model("cpuMargin: 10\n", &[("A", "ram: \"1 kB\"\n"), ("B", "wcet: \"O(1)\"\n")], &[]);
    assert!(codes(&untimed, "E692").is_empty(), "no period, or an opaque wcet");
}

// ── REQ-TRS-BUD-005: wcet against deadline ─────────────────────────────────

#[test]
fn wcet_above_the_deadline_is_e693_naming_both() {
    let els = model("", &[("A", "wcet: \"3 ms\"\ndeadline: \"2 ms\"\n")], &[]);
    let e = codes(&els, "E693");
    assert_eq!(e.len(), 1, "{e:?}");
    assert!(e[0].1.contains("Sw::A") && e[0].1.contains("3") && e[0].1.contains("2"), "{e:?}");
    assert!(codes(&model("", &[("A", "wcet: \"2 ms\"\ndeadline: \"2 ms\"\n")], &[]), "E693").is_empty(), "equal passes");
    assert!(codes(&model("", &[("A", "wcet: \"9 ms\"\n")], &[]), "E693").is_empty(), "only one declared");
    assert!(codes(&model("", &[("A", "wcet: \"O(1)\"\ndeadline: \"2 ms\"\n")], &[]), "E693").is_empty(), "an opaque claim");
}

// ── REQ-TRS-BUD-004: the fields ────────────────────────────────────────────

#[test]
fn durations_and_priority_are_accepted() {
    for fm in ["period: \"10 ms\"\n", "period: \"120 us\"\ndeadline: \"1 ms\"\n", "period: \"0.5 s\"\npriority: 3\n", "priority: 0\n"] {
        let els = model("", &[("A", fm)], &[]);
        assert!(codes(&els, "E691").is_empty(), "{fm:?}: {:?}", validate(&els).findings);
    }
}

#[test]
fn a_malformed_duration_or_priority_is_e691() {
    for fm in ["period: \"10\"\n", "period: \"often\"\n", "period: \"0 ms\"\n", "period: \"-5 ms\"\n", "deadline: \"3 V\"\n", "priority: high\n", "priority: 1.5\n", "period: 10\n"] {
        let els = model("", &[("A", fm)], &[]);
        let e = codes(&els, "E691");
        assert_eq!(e.len(), 1, "{fm:?}: {:?}", validate(&els).findings);
        assert!(e[0].0.ends_with("A.md"), "{e:?}");
    }
}

#[test]
fn the_fields_are_unrecognised_on_other_elements() {
    let r = tempdir();
    write(&r, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(&r, "R.md", "---\ntype: Requirement\nid: REQ-CPU-001\nname: r\nstatus: draft\nreqDomain: software\nperiod: \"1 ms\"\npriority: 1\ncpuMargin: 70\nscheduling: edf\n---\n\nThe system shall work.\n");
    let w = codes(&walk_model(&r).unwrap(), "W047");
    for f in ["period", "priority", "cpuMargin", "scheduling"] {
        assert!(w.iter().any(|(_, m)| m.contains(&format!("'{f}'"))), "{f}: {w:?}");
    }
}

#[test]
fn nothing_declared_means_nothing_new() {
    let els = model("", &[("A", "")], &[]);
    for c in ["E691", "E692", "E693"] {
        assert!(codes(&els, c).is_empty(), "{c}");
    }
}
