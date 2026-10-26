//! TC-TRS-ARCH-004 / GH #369 acceptance on the grid-tie model: the hardware-trip independence rule
//! `AR-GTI-001` passes today and fails, with the path, once the trip is wired to the firmware.

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
    let dst = std::env::temp_dir().join(format!("syscribe-archgti-{}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst))).join("model_gti");
    copy_dir(&Path::new(env!("CARGO_MANIFEST_DIR")).join("../../model_gti"), &dst);
    dst
}

fn validate(root: &Path) -> (String, i32) {
    let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(root).arg("validate").output().unwrap();
    (format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr)), o.status.code().unwrap_or(-1))
}

#[test]
fn the_hardware_trip_rule_passes_on_the_model_as_it_is() {
    let m = gti_copy();
    assert!(m.join("System/Rules/AR-GTI-001.md").exists());
    let (o, c) = validate(&m);
    assert_eq!(c, 0, "{o}");
    assert!(!o.contains("E660"), "{o}");
}

#[test]
fn it_fails_with_the_path_when_the_firmware_is_wired_into_the_trip() {
    let m = gti_copy();
    let fw = m.join("System/Software/Firmware.md");
    let s = std::fs::read_to_string(&fw).unwrap().replacen("features:\n", "features:\n  - {name: tripOut, type: Port, direction: out}\n", 1);
    std::fs::write(&fw, s).unwrap();
    let top = m.join("System/GridTieInverter.md");
    let s = std::fs::read_to_string(&top).unwrap().replacen("connections:\n", "connections:\n  - {from: firmware.tripOut, to: powerStage.hwDisable, flowKind: protection}\n", 1);
    std::fs::write(&top, s).unwrap();
    let (o, c) = validate(&m);
    assert_eq!(c, 1, "{o}");
    let hits: Vec<&str> = o.lines().filter(|l| l.contains("E660")).collect();
    assert_eq!(hits.len(), 1, "one wiring mistake is one finding, not one per contained component: {o}");
    assert!(hits[0].contains("System::Hardware::PowerStage → System::Software::Firmware") && hits[0].contains("AR-GTI-001"), "{}", hits[0]);
}
