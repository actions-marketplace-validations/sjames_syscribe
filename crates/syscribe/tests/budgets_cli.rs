//! TC-TRS-BUD-003 / REQ-TRS-BUD-003 (GH #368): `syscribe budgets` and the `[budgets] margin` project default.
//!
//! Written before the implementation.

use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

fn model(toml: &str, hw_extra: &str) -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let r = std::env::temp_dir().join(format!("syscribe-budcli-{}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst))).join("model");
    let w = |rel: &str, c: &str| {
        let p = r.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, c).unwrap();
    };
    w("_index.md", "---\ntype: Package\nname: Root\n---\n");
    w("Hw/_index.md", "---\ntype: Package\nname: Hw\n---\n");
    w("Sw/_index.md", "---\ntype: Package\nname: Sw\n---\n");
    w("Hw/Mcu.md", &format!("---\ntype: PartDef\nname: Mcu\ndomain: hardware\nramCapacity: \"100 kB\"\nflashCapacity: \"512 kB\"\n{hw_extra}---\n\nM.\n"));
    w("Sw/A.md", "---\ntype: PartDef\nname: A\ndomain: software\nallocatedTo: [Hw::Mcu]\nram: \"50 kB\"\nflash: \"200 kB\"\n---\n\nA.\n");
    w("Sw/B.md", "---\ntype: PartDef\nname: B\ndomain: software\nallocatedTo: [Hw::Mcu]\nram: \"29 kB\"\nstack: \"1 kB\"\n---\n\nB.\n");
    if !toml.is_empty() {
        w(".syscribe.toml", toml);
    }
    r
}

fn run(root: &PathBuf, args: &[&str]) -> (String, i32) {
    let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(root).args(args).output().unwrap();
    (format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr)), o.status.code().unwrap_or(-1))
}

fn rows(root: &PathBuf) -> Vec<serde_json::Value> {
    let (o, c) = run(root, &["budgets", "--json"]);
    assert_eq!(c, 0, "{o}");
    let v: serde_json::Value = serde_json::from_str(&o).unwrap_or_else(|e| panic!("{e}: {o}"));
    v["budgets"].as_array().unwrap_or_else(|| panic!("{v}")).clone()
}

fn row<'a>(rows: &'a [serde_json::Value], resource: &str) -> &'a serde_json::Value {
    rows.iter().find(|r| r["resource"] == resource).unwrap_or_else(|| panic!("{resource} in {rows:?}"))
}

#[test]
fn the_report_gives_used_capacity_limit_and_contributors_largest_first() {
    let r = model("", "margin: 80\n");
    let rows = rows(&r);
    let ram = row(&rows, "ram");
    assert_eq!(ram["element"], "Hw::Mcu");
    assert_eq!(ram["usedBytes"], 80_000, "50 + 29 + 1 stack: {ram}");
    assert_eq!(ram["capacityBytes"], 100_000);
    assert_eq!(ram["marginPercent"], 80);
    assert_eq!(ram["limitBytes"], 80_000);
    assert!((ram["utilisation"].as_f64().unwrap() - 0.8).abs() < 1e-9, "{ram}");
    let c = ram["contributors"].as_array().unwrap();
    assert_eq!((c[0]["element"].as_str(), c[0]["bytes"].as_u64()), (Some("Sw::A"), Some(50_000)), "{ram}");
    assert_eq!(c[1]["element"], "Sw::B");
    let flash = row(&rows, "flash");
    assert_eq!(flash["usedBytes"], 200_000, "{flash}");
    assert_eq!(flash["marginPercent"], 80, "one margin per part, for every resource");
}

#[test]
fn the_project_margin_applies_unless_the_part_sets_its_own() {
    let project = model("[budgets]\nmargin = 70\n", "");
    let (o, c) = run(&project, &["validate"]);
    assert_eq!(c, 1, "80 kB of RAM against a 70 kB limit: {o}");
    assert!(o.contains("E690"), "{o}");
    assert_eq!(row(&rows(&project), "ram")["marginPercent"], 70);
    let own = model("[budgets]\nmargin = 70\n", "margin: 90\n");
    assert_eq!(run(&own, &["validate"]).1, 0);
    assert_eq!(row(&rows(&own), "ram")["marginPercent"], 90);
}

#[test]
fn the_text_report_is_a_table() {
    let (o, c) = run(&model("", ""), &["budgets"]);
    assert_eq!(c, 0, "{o}");
    assert!(o.contains("Hw::Mcu") && o.contains("ram") && o.contains("Sw::A"), "{o}");
}

#[test]
fn nothing_declared_says_so_and_exits_zero() {
    let r = model("", "");
    for f in ["Hw/Mcu.md", "Sw/A.md", "Sw/B.md"] {
        let p = r.join(f);
        let s = std::fs::read_to_string(&p).unwrap();
        let kept: Vec<&str> = s.lines().filter(|l| !(l.starts_with("ram") || l.starts_with("flash") || l.starts_with("stack"))).collect();
        std::fs::write(p, kept.join("\n") + "\n").unwrap();
    }
    let (o, c) = run(&r, &["budgets"]);
    assert_eq!(c, 0, "{o}");
    assert!(o.to_lowercase().contains("no"), "{o}");
    let (j, _) = run(&r, &["budgets", "--json"]);
    let v: serde_json::Value = serde_json::from_str(&j).unwrap();
    assert_eq!(v["budgets"], serde_json::json!([]));
}

// ── REQ-TRS-BUD-006: the cpu part of the report ─────────────────────────────

fn cpu_model(hw_extra: &str) -> PathBuf {
    let r = model("", hw_extra);
    let p = r.join("Sw/A.md");
    let s = std::fs::read_to_string(&p).unwrap().replacen("ram: \"50 kB\"\n", "ram: \"50 kB\"\nwcet: \"4 ms\"\nperiod: \"10 ms\"\n", 1);
    std::fs::write(p, s).unwrap();
    let p = r.join("Sw/B.md");
    let s = std::fs::read_to_string(&p).unwrap().replacen("ram: \"29 kB\"\n", "ram: \"29 kB\"\nwcet: \"10 ms\"\nperiod: \"100 ms\"\n", 1);
    std::fs::write(p, s).unwrap();
    r
}

#[test]
fn the_json_report_carries_cpu_utilisation_with_its_tasks() {
    let r = cpu_model("cpuMargin: 70\nscheduling: rate-monotonic\n");
    let (o, c) = run(&r, &["budgets", "--json"]);
    assert_eq!(c, 0, "{o}");
    let v: serde_json::Value = serde_json::from_str(&o).unwrap();
    let cpu = v["cpu"].as_array().unwrap_or_else(|| panic!("{v}"));
    assert_eq!(cpu.len(), 1, "{v}");
    let row = &cpu[0];
    assert_eq!(row["element"], "Hw::Mcu");
    assert_eq!(row["scheduling"], "rate-monotonic");
    assert_eq!(row["marginPercent"], 70);
    assert_eq!(row["tasks"].as_array().unwrap().len(), 2, "{row}");
    assert!((row["utilisation"].as_f64().unwrap() - 0.5).abs() < 1e-9, "40 % + 10 %: {row}");
    assert!((row["boundPercent"].as_f64().unwrap() - 82.84).abs() < 0.01, "{row}");
    assert_eq!(row["tasks"][0]["element"], "Sw::A", "largest first: {row}");
}

#[test]
fn a_part_with_no_cpu_budget_has_no_cpu_row_and_the_memory_rows_are_unchanged() {
    let r = cpu_model("");
    let (o, _) = run(&r, &["budgets", "--json"]);
    let v: serde_json::Value = serde_json::from_str(&o).unwrap();
    assert_eq!(v["cpu"], serde_json::json!([]));
    assert!(!v["budgets"].as_array().unwrap().is_empty());
}

#[test]
fn validate_reports_the_overload() {
    let r = cpu_model("cpuMargin: 40\n");
    let (o, c) = run(&r, &["validate"]);
    assert_eq!(c, 1, "50 % against a 40 % limit: {o}");
    assert!(o.contains("E692"), "{o}");
}
