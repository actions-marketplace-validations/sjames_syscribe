//! `budgets` — memory use against capacity per hardware part (`REQ-TRS-BUD-003`, GH #368). It shares its computation
//! with the `E690`/`E691` checks of `validate`.

use serde_json::json;
use syscribe_model::budgets::{compute, human, project_margin};
use syscribe_model::element::RawElement;
use syscribe_model::resolver::Resolver;

/// A whole number as an integer, else as it is (byte counts and percentages are usually whole).
fn num(x: f64) -> serde_json::Value {
    if x.fract() == 0.0 && x >= 0.0 { json!(x as u64) } else { json!(x) }
}

pub fn cmd_budgets(elements: &[RawElement], model_root: &std::path::Path, rest: &[String]) -> i32 {
    let resolver = Resolver::new(elements);
    let (rows, problems) = compute(elements, &resolver, project_margin(Some(model_root)));
    for (_, _, file, msg) in &problems {
        eprintln!("{file}: {msg}");
    }
    if rest.iter().any(|a| a == "--json") {
        let out: Vec<_> = rows
            .iter()
            .map(|r| {
                json!({
                    "element": r.element,
                    "resource": r.resource,
                    "usedBytes": num(r.used),
                    "capacityBytes": num(r.capacity),
                    "marginPercent": num(r.margin),
                    "limitBytes": num(r.limit()),
                    "utilisation": r.utilisation(),
                    "contributors": r.contributors.iter().map(|(e, b)| json!({"element": e, "bytes": num(*b)})).collect::<Vec<_>>(),
                })
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&json!({"budgets": out})).unwrap());
    } else if rows.is_empty() {
        println!("No hardware part declares a memory capacity (`ramCapacity`, `flashCapacity`, `nvmCapacity`).");
    } else {
        println!("| Part | Resource | Used | Capacity | Margin | Limit | Use | Largest contributors |\n|---|---|---|---|---|---|---|---|");
        for r in &rows {
            let top = r.contributors.iter().take(3).map(|(e, b)| format!("{e} {}", human(*b))).collect::<Vec<_>>().join(", ");
            let over = if r.used > r.limit() + 1e-9 { " **over**" } else { "" };
            println!(
                "| {} | {} | {}{} | {} | {}% | {} | {:.0}% | {} |",
                r.element,
                r.resource,
                human(r.used),
                over,
                human(r.capacity),
                syscribe_model::expr::format_number(r.margin),
                human(r.limit()),
                r.utilisation() * 100.0,
                top
            );
        }
    }
    0
}
