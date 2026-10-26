//! TC-TRS-META-030 / REQ-TRS-META-030 (GH #377): a `MetadataDef` can specialise another.
//!
//! Written before the implementation: `supertype:` on a `MetadataDef` must give specialisation
//! semantics — inherited tagged values, checks and applicability — and the `E328`/`E329` findings.

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
    let p = std::env::temp_dir().join(format!("syscribe-metaprof-{}-{}", std::process::id(), N.fetch_add(1, Ordering::Relaxed)));
    std::fs::create_dir_all(&p).unwrap();
    p
}

fn model(files: &[(&str, &str)]) -> Vec<RawElement> {
    let r = tempdir();
    write(&r, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(&r, "B/_index.md", "---\ntype: Package\nname: B\n---\n");
    for (p, c) in files {
        write(&r, p, c);
    }
    walk_model(&r).unwrap()
}

fn codes(els: &[RawElement], code: &str) -> Vec<(String, String)> {
    validate(els).findings.iter().filter(|f| f.code == code).map(|f| (f.file.clone(), f.message.clone())).collect()
}

/// `Safety`: integrity 0..4 (required), goal; `ASIL_D :> Safety` adds `rationale`.
const SAFETY: &str = "---\ntype: MetadataDef\nname: Safety\nannotates: [PartDef, ActionDef]\nfeatures:\n  - name: integrity\n    typedBy: ScalarValues::Integer\n    range: \"0..4\"\n    multiplicity: \"1\"\n  - name: goal\n    typedBy: ScalarValues::String\n    enumValues: [SG-1, SG-2]\n---\n";
const ASIL_D: &str = "---\ntype: MetadataDef\nname: ASIL_D\nsupertype: B::Safety\nfeatures:\n  - name: rationale\n    typedBy: ScalarValues::String\n---\n";

fn item(application: &str) -> (String, String) {
    ("B/Item.md".into(), format!("---\ntype: PartDef\nname: Item\nmetadata:\n{application}---\n"))
}

fn with_item(extra: &[(&str, &str)], application: &str) -> Vec<RawElement> {
    let (p, c) = item(application);
    let mut files: Vec<(&str, &str)> = extra.to_vec();
    files.push((&p, &c));
    // `files` borrows p/c; build before they drop.
    model(&files)
}

// ── inherited values are checked ───────────────────────────────────────────

#[test]
fn an_inherited_value_is_type_checked_on_a_sub_definition_application() {
    let els = with_item(&[("B/Safety.md", SAFETY), ("B/ASIL_D.md", ASIL_D)], "  - type: B::ASIL_D\n    integrity: banana\n");
    assert_eq!(codes(&els, "E322").len(), 1, "{:?}", validate(&els).findings);
}

#[test]
fn an_inherited_range_and_enumeration_apply() {
    let els = with_item(&[("B/Safety.md", SAFETY), ("B/ASIL_D.md", ASIL_D)], "  - type: B::ASIL_D\n    integrity: 9\n    goal: SG-9\n");
    assert_eq!(codes(&els, "E323").len(), 1);
    assert_eq!(codes(&els, "E324").len(), 1);
}

#[test]
fn an_inherited_required_value_that_is_omitted_is_e325() {
    let els = with_item(&[("B/Safety.md", SAFETY), ("B/ASIL_D.md", ASIL_D)], "  - type: B::ASIL_D\n    rationale: because\n");
    assert_eq!(codes(&els, "E325").len(), 1);
}

#[test]
fn inherited_keys_are_not_undeclared_but_unknown_keys_still_are() {
    let els = with_item(&[("B/Safety.md", SAFETY), ("B/ASIL_D.md", ASIL_D)], "  - type: B::ASIL_D\n    integrity: 1\n    goal: SG-1\n    rationale: r\n    bogus: 1\n");
    let w = codes(&els, "W045");
    assert_eq!(w.len(), 1, "{w:?}");
    assert!(w[0].1.contains("bogus"), "{w:?}");
}

#[test]
fn a_check_declared_on_the_supertype_runs_on_a_sub_definition_application() {
    let safety = SAFETY.replace("features:", "constraints:\n  - 'integrity >= 2'\nfeatures:");
    let els = with_item(&[("B/Safety.md", &safety), ("B/ASIL_D.md", ASIL_D)], "  - type: B::ASIL_D\n    integrity: 1\n");
    let e = codes(&els, "E327");
    assert_eq!(e.len(), 1, "{:?}", validate(&els).findings);
    assert!(e[0].1.contains("Safety"), "{e:?}");
}

#[test]
fn a_default_declared_on_the_supertype_is_inherited() {
    let safety = SAFETY.replace("multiplicity: \"1\"", "multiplicity: \"1\"\n    default: 2");
    let els = with_item(&[("B/Safety.md", &safety), ("B/ASIL_D.md", ASIL_D)], "  - type: B::ASIL_D\n    rationale: r\n");
    assert!(codes(&els, "E325").is_empty(), "the inherited default satisfies the required value");
}

// ── has() and meta() follow specialisation ────────────────────────────────

#[test]
fn has_and_meta_see_a_sub_definition_application_as_the_supertype() {
    let reg = "---\ntype: MetadataDef\nname: Regulatory\nconstraints:\n  - 'has(Safety) and meta(Safety).integrity >= 2'\n---\n";
    let ok = with_item(&[("B/Safety.md", SAFETY), ("B/ASIL_D.md", ASIL_D), ("B/Regulatory.md", reg)], "  - B::Regulatory\n  - type: B::ASIL_D\n    integrity: 3\n");
    assert!(codes(&ok, "E327").is_empty(), "{:?}", validate(&ok).findings);
    let low = with_item(&[("B/Safety.md", SAFETY), ("B/ASIL_D.md", ASIL_D), ("B/Regulatory.md", reg)], "  - B::Regulatory\n  - type: B::ASIL_D\n    integrity: 1\n");
    assert_eq!(codes(&low, "E327").len(), 1);
}

// ── narrowing ──────────────────────────────────────────────────────────────

#[test]
fn a_sub_definition_may_narrow_a_range_and_an_enumeration() {
    let narrow = "---\ntype: MetadataDef\nname: ASIL_D\nsupertype: B::Safety\nfeatures:\n  - name: integrity\n    typedBy: ScalarValues::Integer\n    range: \"3..4\"\n  - name: goal\n    typedBy: ScalarValues::String\n    enumValues: [SG-1]\n---\n";
    let els = with_item(&[("B/Safety.md", SAFETY), ("B/ASIL_D.md", narrow)], "  - type: B::ASIL_D\n    integrity: 3\n    goal: SG-1\n");
    assert!(codes(&els, "E329").is_empty(), "{:?}", validate(&els).findings);
    // The narrowed range is the one applied.
    let bad = with_item(&[("B/Safety.md", SAFETY), ("B/ASIL_D.md", narrow)], "  - type: B::ASIL_D\n    integrity: 1\n");
    assert_eq!(codes(&bad, "E323").len(), 1);
}

#[test]
fn widening_a_range_or_an_enumeration_is_e329_on_the_sub_definition() {
    let wide = "---\ntype: MetadataDef\nname: ASIL_D\nsupertype: B::Safety\nfeatures:\n  - name: integrity\n    typedBy: ScalarValues::Integer\n    range: \"0..9\"\n  - name: goal\n    typedBy: ScalarValues::String\n    enumValues: [SG-1, SG-2, SG-3]\n---\n";
    let els = model(&[("B/Safety.md", SAFETY), ("B/ASIL_D.md", wide)]);
    let e = codes(&els, "E329");
    assert_eq!(e.len(), 2, "{e:?}");
    assert!(e.iter().all(|(f, _)| f.ends_with("ASIL_D.md")), "{e:?}");
}

#[test]
fn a_sub_definition_annotating_a_kind_the_parent_does_not_is_e329() {
    let bad = "---\ntype: MetadataDef\nname: ASIL_D\nsupertype: B::Safety\nannotates: [PartDef, PortDef]\n---\n";
    let els = model(&[("B/Safety.md", SAFETY), ("B/ASIL_D.md", bad)]);
    assert_eq!(codes(&els, "E329").len(), 1);
    let ok = bad.replace("[PartDef, PortDef]", "[PartDef]");
    assert!(codes(&model(&[("B/Safety.md", SAFETY), ("B/ASIL_D.md", &ok)]), "E329").is_empty());
}

#[test]
fn a_sub_definition_without_annotates_inherits_the_parents() {
    let port = ("B/Port1.md", "---\ntype: PortDef\nname: Port1\nmetadata:\n  - type: B::ASIL_D\n    integrity: 1\n---\n");
    let els = model(&[("B/Safety.md", SAFETY), ("B/ASIL_D.md", ASIL_D), port]);
    assert_eq!(codes(&els, "E318").len(), 1, "Safety annotates only PartDef and ActionDef");
}

// ── abstract definitions ───────────────────────────────────────────────────

#[test]
fn an_abstract_definition_cannot_be_applied_but_its_specialisation_can() {
    let abs = SAFETY.replace("name: Safety", "name: Safety\nisAbstract: true");
    let direct = with_item(&[("B/Safety.md", &abs), ("B/ASIL_D.md", ASIL_D)], "  - type: B::Safety\n    integrity: 1\n");
    assert_eq!(codes(&direct, "E328").len(), 1);
    let via_sub = with_item(&[("B/Safety.md", &abs), ("B/ASIL_D.md", ASIL_D)], "  - type: B::ASIL_D\n    integrity: 1\n");
    assert!(codes(&via_sub, "E328").is_empty());
}

// ── supertype problems ─────────────────────────────────────────────────────

#[test]
fn an_unresolved_or_non_metadata_supertype_is_e329() {
    let nowhere = "---\ntype: MetadataDef\nname: X\nsupertype: B::Nowhere\n---\n";
    let part = ("B/P.md", "---\ntype: PartDef\nname: P\n---\n");
    let not_meta = "---\ntype: MetadataDef\nname: Y\nsupertype: B::P\n---\n";
    let els = model(&[("B/X.md", nowhere), ("B/P.md", part.1), ("B/Y.md", not_meta)]);
    let e = codes(&els, "E329");
    assert_eq!(e.len(), 2, "{e:?}");
}

#[test]
fn a_supertype_cycle_is_e329_and_does_not_hang() {
    let a = "---\ntype: MetadataDef\nname: A\nsupertype: B::B\n---\n";
    let b = "---\ntype: MetadataDef\nname: B\nsupertype: B::A\n---\n";
    let selfref = "---\ntype: MetadataDef\nname: S\nsupertype: B::S\n---\n";
    let els = with_item(&[("B/A.md", a), ("B/B.md", b), ("B/S.md", selfref)], "  - type: B::A\n");
    let e = codes(&els, "E329");
    assert!(e.len() >= 3, "A, B and S are each on a cycle: {e:?}");
}
