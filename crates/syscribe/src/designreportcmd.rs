//! `design-report <part>` — the detailed-design report of one software part (`REQ-TRS-DREP-001`, GH #371).

use syscribe_model::design_report::{build, render};
use syscribe_model::element::RawElement;
use syscribe_model::resolver::Resolver;
use syscribe_model::results::ResultsData;

pub fn cmd_design_report(elements: &[RawElement], model_root: &std::path::Path, rest: &[String]) -> i32 {
    let Some(part) = rest.iter().find(|a| !a.starts_with("--")) else {
        eprintln!("Error: design-report needs a software part: syscribe design-report <part> [--json]");
        return 1;
    };
    let resolver = Resolver::new(elements);
    let results = ResultsData::load_sidecar(model_root);
    match build(elements, &resolver, results.as_ref(), part) {
        Ok(r) => {
            if rest.iter().any(|a| a == "--json") {
                println!("{}", serde_json::to_string_pretty(&r).unwrap());
            } else {
                print!("{}", render(&r));
            }
            0
        }
        Err(m) => {
            eprintln!("Error: {m}");
            1
        }
    }
}
