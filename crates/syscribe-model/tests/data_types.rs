//! TC-TRS-DATA-001..003 / REQ-TRS-DATA-001..003 (GH #366, item 2): `range:`, `pattern:` and `default:` on an
//! `AttributeDef`, a subtype that stays within them, and values bound to features typed by it.
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
    let p = std::env::temp_dir().join(format!("syscribe-data-{}-{}", std::process::id(), N.fetch_add(1, Ordering::Relaxed)));
    std::fs::create_dir_all(&p).unwrap();
    p
}

/// Package `D` holding `files` (path under `D/`, file content).
fn model(files: &[(&str, &str)]) -> Vec<RawElement> {
    let r = tempdir();
    write(&r, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(&r, "D/_index.md", "---\ntype: Package\nname: D\n---\n");
    for (p, c) in files {
        write(&r, &format!("D/{p}"), c);
    }
    walk_model(&r).unwrap()
}

fn attr(name: &str, fm: &str) -> (String, String) {
    (format!("{name}.md"), format!("---\ntype: AttributeDef\nname: {name}\n{fm}---\n\n{name}.\n"))
}

fn m(defs: &[(String, String)], extra: &[(&str, &str)]) -> Vec<RawElement> {
    let mut files: Vec<(&str, &str)> = defs.iter().map(|(p, c)| (p.as_str(), c.as_str())).collect();
    files.extend_from_slice(extra);
    model(&files)
}

fn codes(els: &[RawElement], code: &str) -> Vec<(String, String)> {
    validate(els).findings.iter().filter(|f| f.code == code).map(|f| (f.file.clone(), f.message.clone())).collect()
}

/// A part `D::User` with one inline feature typed by `ty` and the given `value:` line (YAML scalar text).
fn user(ty: &str, value: &str) -> (&'static str, String) {
    ("User.md", format!("---\ntype: PartDef\nname: User\nfeatures:\n  - {{name: v, typedBy: {ty}, value: {value}}}\n---\n\nU.\n"))
}

fn with_user(defs: &[(String, String)], ty: &str, value: &str) -> Vec<RawElement> {
    let (p, c) = user(ty, value);
    m(defs, &[(p, c.as_str())])
}

// ── REQ-TRS-DATA-001 ───────────────────────────────────────────────────────

#[test]
fn every_range_form_and_a_pattern_validate() {
    for range in ["0..250", "[0, 250]", "(0, 250)", "[0, 250)", "(0, 250]", "-40..85.5", "[ -1 , 1 ]"] {
        let els = m(&[attr("T", &format!("range: \"{range}\"\npattern: \"^[A-Z]+$\"\n"))], &[]);
        assert!(codes(&els, "E680").is_empty(), "{range}: {:?}", validate(&els).findings);
    }
}

#[test]
fn a_malformed_range_or_pattern_is_e680() {
    for fm in ["range: \"abc\"\n", "range: \"5..1\"\n", "range: \"[1,\"\n", "range: \"1..2..3\"\n", "range: \"\"\n", "pattern: \"([\"\n", "range: 7\n"] {
        let els = m(&[attr("T", fm)], &[]);
        let e = codes(&els, "E680");
        assert_eq!(e.len(), 1, "{fm:?}: {:?}", validate(&els).findings);
        assert!(e[0].0.ends_with("T.md"), "reported on the type: {e:?}");
    }
}

#[test]
fn the_fields_are_unrecognised_on_other_elements() {
    let els = model(&[("P.md", "---\ntype: PartDef\nname: P\nrange: \"0..1\"\npattern: \"x\"\ndefault: 1\n---\n\nP.\n")]);
    let w = codes(&els, "W047");
    for f in ["range", "pattern", "default"] {
        assert!(w.iter().any(|(_, m)| m.contains(&format!("'{f}'"))), "{f}: {w:?}");
    }
}

// ── REQ-TRS-DATA-002 ───────────────────────────────────────────────────────

#[test]
fn a_default_outside_the_constraints_is_e681() {
    let bad_num = m(&[attr("Speed", "range: \"0..250\"\ndefault: 300\n")], &[]);
    assert_eq!(codes(&bad_num, "E681").len(), 1);
    let ok = m(&[attr("Speed", "range: \"0..250\"\ndefault: 100\n")], &[]);
    assert!(codes(&ok, "E681").is_empty());
    let bad_pat = m(&[attr("Code", "pattern: \"^[A-Z]{3}$\"\ndefault: \"abcd\"\n")], &[]);
    assert_eq!(codes(&bad_pat, "E681").len(), 1);
    let ok_pat = m(&[attr("Code", "pattern: \"^[A-Z]{3}$\"\ndefault: \"ABC\"\n")], &[]);
    assert!(codes(&ok_pat, "E681").is_empty());
    let not_a_number = m(&[attr("Speed", "range: \"0..250\"\ndefault: fast\n")], &[]);
    assert_eq!(codes(&not_a_number, "E681").len(), 1, "a non-numeric default under a numeric range");
}

#[test]
fn a_subtype_may_narrow_the_range() {
    let base = attr("Speed", "range: \"0..250\"\n");
    for sub in ["100..200", "[0, 250]", "(0, 250)", "0..250"] {
        let els = m(&[base.clone(), attr("Fast", &format!("supertype: D::Speed\nrange: \"{sub}\"\n"))], &[]);
        assert!(codes(&els, "E683").is_empty(), "{sub}: {:?}", validate(&els).findings);
    }
}

#[test]
fn a_subtype_may_not_widen_the_range_or_change_the_pattern() {
    let base = attr("Speed", "range: \"0..250\"\n");
    for sub in ["0..300", "-1..250"] {
        let els = m(&[base.clone(), attr("Fast", &format!("supertype: D::Speed\nrange: \"{sub}\"\n"))], &[]);
        let e = codes(&els, "E683");
        assert_eq!(e.len(), 1, "{sub}: {:?}", validate(&els).findings);
        assert!(e[0].0.ends_with("Fast.md"), "on the subtype: {e:?}");
    }
    // an open end of the supertype is not covered by a closed end of the subtype
    let open = m(&[attr("Speed", "range: \"(0, 250)\"\n"), attr("Fast", "supertype: D::Speed\nrange: \"[0, 250]\"\n")], &[]);
    assert_eq!(codes(&open, "E683").len(), 1);
    let pat = m(&[attr("Code", "pattern: \"^[A-Z]+$\"\n"), attr("Short", "supertype: D::Code\npattern: \"^[A-Z]{3}$\"\n")], &[]);
    assert_eq!(codes(&pat, "E683").len(), 1, "another pattern cannot be shown to narrow");
    let same = m(&[attr("Code", "pattern: \"^[A-Z]+$\"\n"), attr("Same", "supertype: D::Code\npattern: \"^[A-Z]+$\"\n")], &[]);
    assert!(codes(&same, "E683").is_empty());
}

#[test]
fn a_subtype_inherits_the_constraints_including_for_its_default() {
    let els = m(&[attr("Speed", "range: \"0..250\"\n"), attr("Fast", "supertype: D::Speed\ndefault: 300\n")], &[]);
    assert_eq!(codes(&els, "E681").len(), 1, "300 is outside the inherited range");
    let ok = m(&[attr("Speed", "range: \"0..250\"\n"), attr("Fast", "supertype: D::Speed\ndefault: 200\n")], &[]);
    assert!(codes(&ok, "E681").is_empty());
}

// ── REQ-TRS-DATA-003 ───────────────────────────────────────────────────────

#[test]
fn a_bound_value_outside_the_range_is_e682_and_inside_is_quiet() {
    let defs = [attr("Speed", "range: \"0..250\"\n")];
    let bad = with_user(&defs, "D::Speed", "300");
    let e = codes(&bad, "E682");
    assert_eq!(e.len(), 1, "{:?}", validate(&bad).findings);
    assert!(e[0].1.contains("'v'") && e[0].1.contains("300") && e[0].1.contains("0..250"), "{e:?}");
    assert!(e[0].0.ends_with("User.md"), "on the element that binds it: {e:?}");
    for ok in ["130", "0", "250", "\"130\""] {
        assert!(codes(&with_user(&defs, "D::Speed", ok), "E682").is_empty(), "{ok}");
    }
}

#[test]
fn an_open_end_excludes_its_bound() {
    let defs = [attr("Speed", "range: \"(0, 250)\"\n")];
    assert_eq!(codes(&with_user(&defs, "D::Speed", "250"), "E682").len(), 1);
    assert_eq!(codes(&with_user(&defs, "D::Speed", "0"), "E682").len(), 1);
    assert!(codes(&with_user(&defs, "D::Speed", "1"), "E682").is_empty());
}

#[test]
fn a_string_must_match_the_pattern_in_full() {
    let defs = [attr("Code", "pattern: \"[A-Z]{3}\"\n")];
    assert!(codes(&with_user(&defs, "D::Code", "ABC"), "E682").is_empty());
    assert_eq!(codes(&with_user(&defs, "D::Code", "ABCD"), "E682").len(), 1, "an unanchored pattern still has to match the whole value");
    assert_eq!(codes(&with_user(&defs, "D::Code", "xABC"), "E682").len(), 1);
}

#[test]
fn a_unit_is_converted_to_the_types_unit() {
    let defs = [attr("Length", "unit: m\nrange: \"0..100\"\n")];
    assert!(codes(&with_user(&defs, "D::Length", "\"500 mm\""), "E682").is_empty(), "0.5 m");
    assert!(codes(&with_user(&defs, "D::Length", "\"99 m\""), "E682").is_empty());
    assert_eq!(codes(&with_user(&defs, "D::Length", "\"150 m\""), "E682").len(), 1);
    assert_eq!(codes(&with_user(&defs, "D::Length", "\"0.2 km\""), "E682").len(), 1, "200 m");
    assert_eq!(codes(&with_user(&defs, "D::Length", "\"5 s\""), "E682").len(), 1, "another dimension");
}

#[test]
fn an_attribute_element_and_an_inherited_constraint_are_checked_too() {
    let defs = [attr("Speed", "range: \"0..250\"\n"), attr("Fast", "supertype: D::Speed\n")];
    let els = m(&defs, &[("a.md", "---\ntype: Attribute\nname: a\ntypedBy: D::Speed\nvalue: \"300\"\n---\n\nA.\n")]);
    let e = codes(&els, "E682");
    assert_eq!(e.len(), 1, "{:?}", validate(&els).findings);
    assert!(e[0].0.ends_with("a.md"), "{e:?}");
    assert_eq!(codes(&with_user(&defs, "D::Fast", "300"), "E682").len(), 1, "Fast inherits Speed's range");
}

#[test]
fn nothing_declared_means_nothing_new() {
    let none = with_user(&[attr("Plain", "")], "D::Plain", "99999");
    let builtin = with_user(&[], "ScalarValues::Integer", "99999");
    let no_value = m(&[attr("Speed", "range: \"0..250\"\n")], &[("U2.md", "---\ntype: PartDef\nname: U2\nfeatures:\n  - {name: v, typedBy: D::Speed}\n---\n\nU.\n")]);
    for els in [&none, &builtin, &no_value] {
        for c in ["E680", "E681", "E682", "E683"] {
            assert!(codes(els, c).is_empty(), "{c}: {:?}", validate(els).findings);
        }
    }
}
