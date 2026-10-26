//! TC-TRS-BUD-001, -002 / REQ-TRS-BUD-001, -002 (GH #368): static memory declared on software, capacity and margin
//! on hardware, and the roll-up check in `validate`.
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
    let p = std::env::temp_dir().join(format!("syscribe-bud-{}-{}", std::process::id(), N.fetch_add(1, Ordering::Relaxed)));
    std::fs::create_dir_all(&p).unwrap();
    p
}

/// `Hw::Mcu` (hardware, frontmatter `hw`) and software parts `Sw::<name>` allocated to it (`allocatedTo:`), each with
/// the given extra frontmatter lines; `extra` adds more files.
fn model(hw: &str, sw: &[(&str, &str)], extra: &[(&str, &str)]) -> Vec<RawElement> {
    let r = tempdir();
    write(&r, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    for p in ["Hw", "Sw"] {
        write(&r, &format!("{p}/_index.md"), &format!("---\ntype: Package\nname: {p}\n---\n"));
    }
    write(&r, "Hw/Mcu.md", &format!("---\ntype: PartDef\nname: Mcu\ndomain: hardware\n{hw}---\n\nMcu.\n"));
    for (name, fm) in sw {
        write(&r, &format!("Sw/{name}.md"), &format!("---\ntype: PartDef\nname: {name}\ndomain: software\nallocatedTo: [Hw::Mcu]\n{fm}---\n\n{name}.\n"));
    }
    for (p, c) in extra {
        write(&r, p, c);
    }
    walk_model(&r).unwrap()
}

fn codes(els: &[RawElement], code: &str) -> Vec<(String, String)> {
    validate(els).findings.iter().filter(|f| f.code == code).map(|f| (f.file.clone(), f.message.clone())).collect()
}

const CAP: &str = "ramCapacity: \"100 kB\"\nmargin: 80\n";

// ── REQ-TRS-BUD-002: the roll-up check ─────────────────────────────────────

#[test]
fn use_within_the_margin_passes_and_above_it_is_e690_naming_the_contributors() {
    let ok = model(CAP, &[("A", "ram: \"50 kB\"\n"), ("B", "ram: \"29 kB\"\n")], &[]);
    assert!(codes(&ok, "E690").is_empty(), "79 of an 80 limit: {:?}", validate(&ok).findings);
    let over = model(CAP, &[("A", "ram: \"50 kB\"\n"), ("B", "ram: \"31 kB\"\n")], &[]);
    let e = codes(&over, "E690");
    assert_eq!(e.len(), 1, "{e:?}");
    assert!(e[0].0.ends_with("Mcu.md"), "reported on the hardware part: {e:?}");
    let m = &e[0].1;
    assert!(m.contains("Hw::Mcu") && m.contains("ram") && m.contains("81") && m.contains("80"), "{m}");
    assert!(m.find("Sw::A").unwrap() < m.find("Sw::B").unwrap(), "largest contributor first: {m}");
}

#[test]
fn use_equal_to_the_limit_passes() {
    let els = model(CAP, &[("A", "ram: \"80 kB\"\n")], &[]);
    assert!(codes(&els, "E690").is_empty());
}

#[test]
fn the_default_margin_is_the_whole_capacity() {
    let hw = "ramCapacity: \"100 kB\"\n";
    assert!(codes(&model(hw, &[("A", "ram: \"100 kB\"\n")], &[]), "E690").is_empty());
    assert_eq!(codes(&model(hw, &[("A", "ram: \"101 kB\"\n")], &[]), "E690").len(), 1);
}

#[test]
fn stack_counts_toward_ram_and_flash_is_its_own_resource() {
    let els = model(CAP, &[("A", "ram: \"60 kB\"\nstack: \"25 kB\"\nflash: \"1 MB\"\n")], &[]);
    let e = codes(&els, "E690");
    assert_eq!(e.len(), 1, "85 kB of RAM; flash has no capacity: {e:?}");
    assert!(e[0].1.contains("ram"), "{e:?}");
    let both = model(&format!("{CAP}flashCapacity: \"512 kB\"\n"), &[("A", "ram: \"10 kB\"\nflash: \"900 kB\"\n")], &[]);
    let e = codes(&both, "E690");
    assert_eq!(e.len(), 1, "{e:?}");
    assert!(e[0].1.contains("flash"), "{e:?}");
}

fn firmware_model(mppt_ram: &str, also_allocated_directly: bool) -> Vec<RawElement> {
    let r = tempdir();
    write(&r, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(&r, "Hw/_index.md", "---\ntype: Package\nname: Hw\n---\n");
    write(&r, "Sw/_index.md", "---\ntype: Package\nname: Sw\n---\n");
    write(&r, "Hw/Mcu.md", "---\ntype: PartDef\nname: Mcu\ndomain: hardware\nramCapacity: \"100 kB\"\nmargin: 80\n---\n\nM.\n");
    let direct = if also_allocated_directly { "allocatedTo: [Hw::Mcu]\n" } else { "" };
    write(&r, "Sw/Sup.md", &format!("---\ntype: PartDef\nname: Sup\ndomain: software\n{direct}ram: \"20 kB\"\n---\n\nS.\n"));
    write(&r, "Sw/Mppt.md", &format!("---\ntype: PartDef\nname: Mppt\ndomain: software\nram: \"{mppt_ram}\"\n---\n\nM.\n"));
    write(&r, "Sw/Firmware.md", "---\ntype: PartDef\nname: Firmware\ndomain: software\nallocatedTo: [Hw::Mcu]\nram: \"10 kB\"\nfeatures:\n  - {name: sup, typedBy: Sw::Sup}\n  - {name: mppt, typedBy: Sw::Mppt}\n---\n\nF.\n");
    walk_model(&r).unwrap()
}

#[test]
fn what_an_allocated_composite_contains_counts_and_each_part_counts_once() {
    // 10 (Firmware) + 20 (Sup) + 50 (Mppt) = 80: the limit, so it passes
    assert!(codes(&firmware_model("50 kB", false), "E690").is_empty());
    // one kB more is over, and the contributors are the composite and its parts
    let e = codes(&firmware_model("51 kB", false), "E690");
    assert_eq!(e.len(), 1, "{e:?}");
    for c in ["Sw::Firmware", "Sw::Sup", "Sw::Mppt"] {
        assert!(e[0].1.contains(c), "{c} contributes: {e:?}");
    }
    // Sup is also allocated directly, but is one component: still 80, still passing
    assert!(codes(&firmware_model("50 kB", true), "E690").is_empty(), "not counted twice");
}

#[test]
fn every_allocation_form_counts() {
    let by_from = model(
        CAP,
        &[],
        &[("Sw/X.md", "---\ntype: PartDef\nname: X\ndomain: software\nram: \"90 kB\"\n---\n\nX.\n"), ("Hw/Mcu.md", "---\ntype: PartDef\nname: Mcu\ndomain: hardware\nramCapacity: \"100 kB\"\nmargin: 80\nallocatedFrom: [Sw::X]\n---\n\nM.\n")],
    );
    assert_eq!(codes(&by_from, "E690").len(), 1, "allocatedFrom on the target");
    let by_element = model(
        CAP,
        &[],
        &[
            ("Sw/X.md", "---\ntype: PartDef\nname: X\ndomain: software\nram: \"90 kB\"\n---\n\nX.\n"),
            ("Alloc/_index.md", "---\ntype: Package\nname: Alloc\n---\n"),
            ("Alloc/A.md", "---\ntype: Allocation\nname: A\nfeatures:\n  - {allocatedFrom: Sw::X, allocatedTo: Hw::Mcu}\n---\n\nA.\n"),
        ],
    );
    assert_eq!(codes(&by_element, "E690").len(), 1, "an Allocation element");
}

#[test]
fn a_part_with_no_capacity_is_not_checked() {
    let els = model("", &[("A", "ram: \"999 MB\"\n")], &[]);
    assert!(codes(&els, "E690").is_empty());
}

// ── REQ-TRS-BUD-001: the fields ────────────────────────────────────────────

#[test]
fn bytes_and_unit_bearing_sizes_are_accepted() {
    for v in ["1024", "\"1024\"", "\"48 KiB\"", "\"1.5 MiB\"", "\"64 kB\"", "\"2 GB\"", "\"512 B\"", "\"3 MiB\""] {
        let els = model("", &[("A", &format!("ram: {v}\n"))], &[]);
        assert!(codes(&els, "E691").is_empty(), "{v}: {:?}", validate(&els).findings);
    }
}

#[test]
fn an_ambiguous_unknown_or_negative_size_is_e691() {
    for v in ["\"48 KB\"", "\"48 kb\"", "\"lots\"", "\"-5 kB\"", "-5", "\"5 furlongs\"", "[1, 2]"] {
        let els = model("", &[("A", &format!("ram: {v}\n"))], &[]);
        let e = codes(&els, "E691");
        assert_eq!(e.len(), 1, "{v}: {:?}", validate(&els).findings);
        assert!(e[0].0.ends_with("A.md"), "{e:?}");
    }
    let cap = model("ramCapacity: \"100 KB\"\n", &[], &[]);
    assert_eq!(codes(&cap, "E691").len(), 1, "a capacity is a size too");
}

#[test]
fn a_margin_outside_0_to_100_is_e691() {
    for v in ["101", "-1", "\"eighty\""] {
        let els = model(&format!("ramCapacity: \"1 kB\"\nmargin: {v}\n"), &[], &[]);
        assert_eq!(codes(&els, "E691").len(), 1, "{v}: {:?}", validate(&els).findings);
    }
    for v in ["0", "80", "100", "\"80%\"", "99.5"] {
        let els = model(&format!("ramCapacity: \"1 kB\"\nmargin: {v}\n"), &[], &[]);
        assert!(codes(&els, "E691").is_empty(), "{v}");
    }
}

#[test]
fn the_fields_are_unrecognised_on_other_elements() {
    let r = tempdir();
    write(&r, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(&r, "R.md", "---\ntype: Requirement\nid: REQ-BUD-001\nname: r\nstatus: draft\nreqDomain: software\nram: \"1 kB\"\nmargin: 80\n---\n\nThe system shall work.\n");
    let els = walk_model(&r).unwrap();
    let w = codes(&els, "W047");
    for f in ["ram", "margin"] {
        assert!(w.iter().any(|(_, m)| m.contains(&format!("'{f}'"))), "{f}: {w:?}");
    }
}

#[test]
fn nothing_declared_means_nothing_new() {
    let els = model("", &[("A", "")], &[]);
    for c in ["E690", "E691"] {
        assert!(codes(&els, c).is_empty(), "{c}");
    }
}
