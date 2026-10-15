//! stdio-subprocess plugin invocation (`ADR-SYS-PLUGIN-002`). `run` invokes
//! one configured plugin live and prints its raw envelope JSON — no merge,
//! no validation. Fast feedback loop for a plugin author, including seeing
//! exactly what came back when it doesn't parse.

use std::path::Path;

use syscribe_model::config::ValidateConfig;
use syscribe_model::plugins::{build_request, find_alias_package, runtime};

/// `plugins run <alias> --dry-run` — currently `--dry-run` is the only
/// supported mode; it's required rather than implied so the call site is
/// unambiguous about what it does (invoke live, print raw, no merge).
pub fn cmd_run(cfg: &ValidateConfig, model_root: &Path, alias: &str, report: Option<&str>) -> i32 {
    let Some(entry) = cfg.plugins.get(alias) else {
        eprintln!("Error: no [plugins.{alias}] entry in .syscribe.toml");
        return 1;
    };
    // A results plugin converts a report; it needs `--report` and no package (REQ-TRS-RESPLUG-001).
    if entry.is_results() {
        let Some(report) = report else {
            eprintln!("Error: [plugins.{alias}] is a results plugin; pass the report to convert: plugins run {alias} --report <file> --dry-run");
            return 1;
        };
        let req = match syscribe_model::plugins::results::build_request(alias, Path::new(report), model_root, None, None, None) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("Error: {e}");
                return 1;
            }
        };
        return match syscribe_model::plugins::results::invoke(entry, &req, model_root) {
            Ok(raw) => {
                println!("{raw}");
                0
            }
            Err(e) => {
                eprintln!("{}", syscribe_model::plugins::results::explain(alias, &e));
                1
            }
        };
    }
    if !entry.is_foreign() {
        eprintln!("E552: [plugins.{alias}] has kind \"{}\", which is neither \"foreign\" nor \"results\"", entry.kind.as_deref().unwrap_or(""));
        return 1;
    }
    if report.is_some() {
        eprintln!("Error: --report applies to a results plugin; [plugins.{alias}] is a foreign-format plugin");
        return 1;
    }
    let Some(pkg) = find_alias_package(model_root, alias) else {
        eprintln!("Error: no package in the model declares foreignFormat: {alias}");
        return 1;
    };

    let req = build_request(&pkg, model_root);

    match runtime::invoke_raw(entry, &req, model_root) {
        Ok(raw_json) => {
            println!("{raw_json}");
            0
        }
        Err(e) => {
            eprintln!("plugin '{alias}' failed: {e}");
            1
        }
    }
}
