//! Results plugins (REQ-TRS-RESPLUG-001, an addendum to `ADR-SYS-PLUGIN-002`): an external converter turns a test
//! report of any format into normalised result records (REQ-TRS-RESREC-001).
//!
//! The protocol is the stdio one of the foreign-format plugins with a second request and response. Syscribe writes
//! one JSON request to the plugin's stdin and reads one JSON object from its stdout:
//!
//! ```json
//! {"protocolVersion":1,"alias":"rig","reportPath":"/abs/log.txt","modelRoot":"/abs/model",
//!  "run":"r1","config":"CONF-X","environment":"TE-HIL-01"}      // run/config/environment only when given
//! {"records":[{"testCase":"TC-1","verdict":"pass"}],"diagnostics":["note"]}
//! ```
//!
//! What comes back is validated by exactly the record validator of the built-in format, so a plugin cannot store
//! anything the built-in format could not.

use std::path::Path;

use serde::{Deserialize, Serialize};

use super::config::PluginEntry;
use super::runtime::{self, PluginError};

/// The protocol version written in every request.
pub const PROTOCOL_VERSION: u32 = 1;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResultsRequest {
    pub protocol_version: u32,
    pub alias: String,
    /// Absolute path of the report to convert.
    pub report_path: String,
    /// Absolute path of the model root.
    pub model_root: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub run: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub environment: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ResultsEnvelope {
    #[serde(default)]
    records: serde_json::Value,
    #[serde(default)]
    diagnostics: Vec<serde_json::Value>,
}

/// What a results plugin handed back, before the records are validated.
pub struct PluginOutput {
    /// The records as a JSON array text, ready for [`crate::results::ResultsData::parse_records`].
    pub records_json: String,
    /// Messages the plugin wants the user to see.
    pub diagnostics: Vec<String>,
}

/// Build the request for `report`, resolving it to absolute paths.
pub fn build_request(alias: &str, report: &Path, model_root: &Path, run: Option<&str>, config: Option<&str>, environment: Option<&str>) -> Result<ResultsRequest, String> {
    let report = std::fs::canonicalize(report).map_err(|e| format!("cannot read the report '{}': {e}", report.display()))?;
    if !report.is_file() {
        return Err(format!("the report '{}' is not a file", report.display()));
    }
    let root = std::fs::canonicalize(model_root).unwrap_or_else(|_| model_root.to_path_buf());
    Ok(ResultsRequest {
        protocol_version: PROTOCOL_VERSION,
        alias: alias.to_string(),
        report_path: report.to_string_lossy().into_owned(),
        model_root: root.to_string_lossy().into_owned(),
        run: run.map(String::from),
        config: config.map(String::from),
        environment: environment.map(String::from),
    })
}

/// Why a results plugin could not be used, worded for the user.
pub fn explain(alias: &str, e: &PluginError) -> String {
    format!("results plugin '{alias}' failed: {e}")
}

/// Run the plugin and return its raw stdout (`plugins run --dry-run` prints this).
pub fn invoke(entry: &PluginEntry, req: &ResultsRequest, model_root: &Path) -> Result<String, PluginError> {
    runtime::invoke_raw(entry, req, model_root)
}

/// Parse a plugin's stdout into records and diagnostics. The envelope must be one object whose `records` is an
/// array; anything else is an error naming what was found.
pub fn parse_envelope(alias: &str, raw: &str) -> Result<PluginOutput, String> {
    // serde would read a bare JSON array as the fields of the struct in order; the envelope is an object.
    let value: serde_json::Value = serde_json::from_str(raw.trim()).map_err(|e| format!("results plugin '{alias}' wrote an invalid envelope (not JSON): {e}"))?;
    if !value.is_object() {
        return Err(format!("results plugin '{alias}' wrote an invalid envelope: expected one JSON object {{\"records\": [...]}}, found {}", match value {
            serde_json::Value::Array(_) => "an array",
            serde_json::Value::Null => "null",
            _ => "a scalar",
        }));
    }
    let env: ResultsEnvelope = serde_json::from_value(value).map_err(|e| format!("results plugin '{alias}' wrote an invalid envelope (expected {{\"records\": [...], \"diagnostics\": [...]}}): {e}"))?;
    if !env.records.is_array() {
        return Err(format!("results plugin '{alias}': \"records\" must be an array of result records"));
    }
    let diagnostics = env
        .diagnostics
        .iter()
        .map(|d| match d {
            serde_json::Value::String(s) => s.clone(),
            other => other.get("message").and_then(|m| m.as_str()).map(String::from).unwrap_or_else(|| other.to_string()),
        })
        .map(|s| crate::results::one_line(&s))
        .collect();
    Ok(PluginOutput { records_json: env.records.to_string(), diagnostics })
}
