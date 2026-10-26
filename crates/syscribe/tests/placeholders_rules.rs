//! GH #268 follow-up: rules that read a placeholder-held typed field do not misreport the base model.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

fn run(root: &Path, args: &[&str]) -> String {
    let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(root).args(args).output().unwrap();
    format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr))
}

fn copy_dir(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for e in std::fs::read_dir(from).unwrap() {
        let e = e.unwrap();
        let p = to.join(e.file_name());
        if e.file_type().unwrap().is_dir() {
            copy_dir(&e.path(), &p);
        } else {
            std::fs::copy(e.path(), p).unwrap();
        }
    }
}

/// The qualification fixture of the quantitative-metrics test, optionally with one event's coverage
/// replaced by `replacement` (a whole `diagnosticCoverage: …` line).
fn metrics_model(replacement: Option<&str>) -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../qual/fixtures/TC-TRS-SAFE-005/metrics");
    let root = std::env::temp_dir().join(format!("syscribe-phrules-{}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst))).join("model");
    copy_dir(&src, &root);
    std::fs::write(root.join("_index.md"), "---\ntype: Package\nname: Root\n---\n").unwrap();
    if let Some(line) = replacement {
        let f = root.join("FTA/FT-MET-001/FTE-MTA-001.md");
        let t = std::fs::read_to_string(&f).unwrap().replace("diagnosticCoverage: 0.99", line);
        std::fs::write(f, t).unwrap();
    }
    root
}

#[test]
fn the_metrics_fixture_normally_misses_its_target() {
    let o = run(&metrics_model(None), &["validate"]);
    assert!(o.contains("W033"), "baseline: SG-MET-001 misses the ASIL D gate: {o}");
}

#[test]
fn a_placeholder_coverage_makes_the_goals_metrics_pending_not_a_miss() {
    let r = metrics_model(Some("diagnosticCoverage: \"{{Features::Hw.dc}}\""));
    let o = run(&r, &["validate"]);
    assert!(!o.lines().any(|l| l.contains("W033") && l.contains("SG-MET-001")), "metrics are per configuration, not a base-model miss: {o}");
    assert!(!o.lines().any(|l| l.contains("W985") && l.contains("FTE-MTA-001")), "{o}");
}

fn plain_model() -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(1000);
    let root = std::env::temp_dir().join(format!("syscribe-phrules-{}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst))).join("model");
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join("_index.md"), "---\ntype: Package\nname: Root\n---\n").unwrap();
    root
}

#[test]
fn a_placeholder_cal_level_is_not_a_cal_inconsistency() {
    let r = plain_model();
    let w = |f: &str, c: &str| std::fs::write(r.join(f), c).unwrap();
    w("DS-PR-001.md", "---\nid: DS-PR-001\ntype: DamageScenario\nname: D\nstatus: draft\ndamageSeverity: severe\n---\n\nb\n");
    w("TS-PR-001.md", "---\nid: TS-PR-001\ntype: ThreatScenario\nname: T\nstatus: draft\ndamageScenarios: [DS-PR-001]\nattackFeasibility: high\nattackVector: network\n---\n\nb\n");
    let goal = |cal: &str| format!("---\nid: CSG-PR-001\ntype: CybersecurityGoal\nname: G\nstatus: draft\nthreatScenarios: [TS-PR-001]\n{cal}securityProperty: integrity\n---\n\nb\n");
    w("CSG-PR-001.md", &goal("calLevel: CAL1\n"));
    assert!(run(&r, &["validate"]).contains("W032"), "baseline: CAL1 is below the expected CAL");
    w("CSG-PR-001.md", &goal("calLevel: \"{{Features::Sec.cal}}\"\n"));
    assert!(!run(&r, &["validate"]).contains("W032"), "judged per configuration, not in the base model");
}

#[test]
fn statistics_name_a_placeholder_level_instead_of_calling_it_qm() {
    let r = plain_model();
    std::fs::write(
        r.join("REQ-PR-001.md"),
        "---\nid: REQ-PR-001\ntype: Requirement\nname: r\nstatus: draft\nreqDomain: software\nasilLevel: \"{{Features::Sec.level}}\"\n---\n\nShall.\n",
    )
    .unwrap();
    let o = run(&r, &["stats", "--group-by", "asilLevel", "--json"]);
    assert!(o.contains("per configuration") && !o.contains("QM/none"), "{o}");
}
