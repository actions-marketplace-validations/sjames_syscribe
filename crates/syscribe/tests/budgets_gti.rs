//! TC-TRS-BUD-002 / GH #368 acceptance on the grid-tie model: `REQ-GTI-SYS-072` ("no more than 80 % of RAM and flash
//! on the selected part") is a checked budget on the STM32G474.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

fn copy_dir(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for e in std::fs::read_dir(from).unwrap() {
        let e = e.unwrap();
        let (src, dst) = (e.path(), to.join(e.file_name()));
        if e.file_type().unwrap().is_dir() {
            copy_dir(&src, &dst);
        } else {
            std::fs::copy(&src, &dst).unwrap();
        }
    }
}

fn gti_copy() -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let dst = std::env::temp_dir().join(format!("syscribe-budgti-{}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst))).join("model_gti");
    copy_dir(&Path::new(env!("CARGO_MANIFEST_DIR")).join("../../model_gti"), &dst);
    dst
}

fn run(root: &Path, args: &[&str]) -> (String, i32) {
    let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(root).args(args).output().unwrap();
    (format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr)), o.status.code().unwrap_or(-1))
}

fn set_firmware_ram(root: &Path, ram: &str) {
    let fw = root.join("System/Software/Firmware.md");
    let s = std::fs::read_to_string(&fw).unwrap().replacen("domain: software\n", &format!("domain: software\nram: \"{ram}\"\n"), 1);
    std::fs::write(fw, s).unwrap();
}

#[test]
fn the_mcus_capacity_and_the_80_percent_margin_are_declared_in_the_model() {
    let m = gti_copy();
    let (o, c) = run(&m, &["budgets", "--json"]);
    assert_eq!(c, 0, "{o}");
    let v: serde_json::Value = serde_json::from_str(&o).unwrap();
    let ram = v["budgets"].as_array().unwrap().iter().find(|r| r["resource"] == "ram").unwrap_or_else(|| panic!("{v}"));
    assert_eq!(ram["element"], "System::Hardware::Stm32G474");
    assert_eq!((ram["capacityBytes"].as_u64(), ram["marginPercent"].as_u64()), (Some(128_000), Some(80)), "{ram}");
    assert_eq!(ram["limitBytes"].as_f64(), Some(102_400.0));
    let (o, c) = run(&m, &["validate"]);
    assert_eq!(c, 0, "no firmware size is declared yet, so nothing is over: {o}");
}

#[test]
fn firmware_within_80_percent_of_ram_passes_and_above_it_names_the_firmware() {
    let ok = gti_copy();
    set_firmware_ram(&ok, "100 kB");
    assert_eq!(run(&ok, &["validate"]).1, 0, "100 of 102.4 kB");
    let over = gti_copy();
    set_firmware_ram(&over, "110 kB");
    let (o, c) = run(&over, &["validate"]);
    assert_eq!(c, 1, "{o}");
    let hit: Vec<&str> = o.lines().filter(|l| l.contains("E690")).collect();
    assert_eq!(hit.len(), 1, "{o}");
    assert!(hit[0].contains("System::Hardware::Stm32G474") && hit[0].contains("System::Software::Firmware") && hit[0].contains("110 kB"), "{}", hit[0]);
}
