//! TC-TRS-BADR-001 / REQ-TRS-BADR-001: `[traceability] require_breakdown_adr = false` switches `E310` off.
//!
//! Written before the implementation.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use syscribe_model::config::ValidateConfig;
use syscribe_model::validator::validate_with_config;
use syscribe_model::walker::walk_model;

fn write(root: &Path, rel: &str, content: &str) {
    let path = root.join(rel);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, content).unwrap();
}

fn tempdir() -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let p = std::env::temp_dir().join(format!("syscribe-badr-{}-{}", std::process::id(), N.fetch_add(1, Ordering::Relaxed)));
    std::fs::create_dir_all(&p).unwrap();
    p
}

/// A parent and a child requirement; the child's `derivedFrom` carries `extra` frontmatter. Returns the
/// findings with `config` as the `.syscribe.toml` (None = no file).
fn findings(config: Option<&str>, extra: &str) -> Vec<(String, String)> {
    let r = tempdir();
    write(&r, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    if let Some(c) = config {
        write(&r, ".syscribe.toml", c);
    }
    let req = |id: &str, body: &str| format!("---\ntype: Requirement\nid: {id}\nname: {id}\nstatus: draft\nreqDomain: software\n{body}---\n\nThe system shall do {id}.\n");
    write(&r, "REQ-001.md", &req("REQ-001", ""));
    write(&r, "REQ-002.md", &req("REQ-002", &format!("derivedFrom:\n  - REQ-001\n{extra}")));
    let els = walk_model(&r).unwrap();
    let cfg = ValidateConfig::with_model_root(&r);
    validate_with_config(&els, &cfg).findings.into_iter().map(|f| (f.code.to_string(), f.message)).collect()
}

fn has(f: &[(String, String)], code: &str) -> bool {
    f.iter().any(|(c, _)| c == code)
}

#[test]
fn default_requires_the_adr() {
    assert!(has(&findings(None, ""), "E310"));
    assert!(has(&findings(Some("[traceability]\nrequire_breakdown_adr = true\n"), ""), "E310"));
}

#[test]
fn disabled_drops_e310() {
    let f = findings(Some("[traceability]\nrequire_breakdown_adr = false\n"), "");
    assert!(!has(&f, "E310"), "{f:?}");
}

#[test]
fn camel_case_key_is_accepted() {
    let f = findings(Some("[traceability]\nrequireBreakdownAdr = false\n"), "");
    assert!(!has(&f, "E310"), "{f:?}");
}

#[test]
fn a_cited_adr_is_still_checked() {
    let f = findings(Some("[traceability]\nrequire_breakdown_adr = false\n"), "breakdownAdr: ADR-NOPE-001\n");
    assert!(has(&f, "E311"), "{f:?}");
}

#[test]
fn malformed_value_warns_and_keeps_the_default() {
    let f = findings(Some("[traceability]\nrequire_breakdown_adr = \"no\"\n"), "");
    assert!(has(&f, "W632"), "{f:?}");
    assert!(has(&f, "E310"), "{f:?}");
}
